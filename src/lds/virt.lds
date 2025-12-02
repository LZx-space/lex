/*
  Linker script for outputting to RISC-V QEMU "virt" machine.
*/

/*
  riscv is the name of the architecture that the linker understands
  for any RISC-V target (64-bit or 32-bit).

  We will further refine this by using -mabi=lp64 and -march=rv64gc
*/
OUTPUT_ARCH( "riscv" )

/*
  将入口点设置为一个名为_start 的符号，它位于boot.S中。
  这本质上是将_start的地址存储为 “入口点”，也就是CPU指令应该开始执行的地方。
  在本脚本的其余部分，我们将把_start放在0x8000_0000的起始位置，因为这是虚拟机和许多RISC-V开发板开始执行的位置。
*/
ENTRY( _start )

/*
  MEMORY部分将说明我们有RAM（随机存取存储器），它包含一个具有 “w”（可写）、“x”（可执行）和 “a”（可分配）属性的区域。
  ORIGIN（起始地址）是物理地址0x8000_0000。查看virt规格或RISC-V HiFive Unleashed的规格说明，就能确定这是我们代码的起始内存地址。
  旁注：在不同的地址可能存在其他启动 ROM（只读存储器），但它们的作用就是引导至这一位置。
  最后，LENGTH=256M告诉链接器我们有256MB的RAM。链接器会对此进行双重检查，以确保所有内容都能容纳其中。
  HiFive Unleashed的RAM实际上比这多得多，但对于虚拟机，我选择了256M，因为我认为目前这个容量的RAM已经足够了。
  我们可以提供其他类型的内存，比如QSPI（四通道串行外设接口或ROM，但在此处的链接器脚本中，我们只告知有RAM。
*/
MEMORY
{
  ram  (wxa) : ORIGIN = 0x80000000, LENGTH = 256M
}

/*
  PHDRS是“程序头”（program headers）的缩写，我们在这里指定了三个：
  text - CPU指令（可执行段）
  data - 全局的、已初始化的变量
  bss  - 全局的、未初始化的变量（所有这些变量都将由boot.S设置为 0）
  PT_LOAD 命令告诉链接器，这些段将从文件加载到内存中。
  实际上，我们可以把所有这些都塞进一个程序头里，但通过将其拆分为三个，我们实际上可以使用其他PT_*命令，
  如PT_DYNAMIC、PT_INTERP、PT_NULL来告诉链接器在哪里可以找到额外的信息。
  不过，就我们的目的而言，每个段都将从程序头加载。
*/
PHDRS
{
  text PT_LOAD;
  data PT_LOAD;
  bss  PT_LOAD;
}

/*
  现在我们将根据内存所在的段来对其进行组织。在汇编语言中，我们可以使用“.section”指令来更改段。
  不过，在C++和Rust中，CPU指令会进入text段，全局常量会进入rodata段，已初始化的全局变量会进入data段，
  而未初始化的全局变量会进入bss段。
*/
SECTIONS
{
  /*
    我们的RAM布局的第一部分将是text段。
    由于我们的CPU指令位于此处，且我们的内存从0x8000_0000开始，因此我们需要让入口点与此处对齐。
  */
  .text : {
    /*
      PROVIDE:
      如果项目中没有其他地方（比如C代码、汇编代码）定义这个符号，就用链接器脚本里的这个定义。
      如果其他地方定义了，就用其他地方的定义（即链接器脚本的定义会被覆盖）。

      这里定义一个名为_text_start的符号，这样我就知道操作系统中文本段的起始位置。
      它不应该移动，但放在这里是为了方便。句点 “.” 告诉链接器将_text_start设置为当前位置（“.”= 当前内存位置）。随着我们添加内容，
      这个当前内存位置会发生变化。
    */
    PROVIDE(_text_start = .);

    /*
      We are going to layout all text sections here, starting with
      .text.init. The asterisk in front of the parentheses means to match
      the .text.init section of ANY object file. Otherwise, we can specify
      which object file should contain the .text.init section, for example,
      boot.o(.text.init) would specifically put the .text.init section of
      our bootloader here.

      Because we might want to change the name of our files, we'll leave it
      with a *.

      Inside the parentheses is the name of the section. I created my own
      called .text.init to make 100% sure that the _start is put right at the
      beginning. The linker will lay this out in the order it receives it:

      .text.init first
      all .text sections next
      any .text.* sections last

      .text.* means to match anything after .text. If we didn't already specify
      .text.init, this would've matched here. The assembler and linker can place
      things in "special" text sections, so we match any we might come across here.
    */
    *(.text.init) *(.text .text.*)

    /*
      Again, with PROVIDE, we're providing a readable symbol called _text_end, which is
      set to the memory address AFTER .text.init, .text, and .text.*'s have been added.
    */
    PROVIDE(_text_end = .);

    /*
      The portion after the right brace is in an odd format. However, this is telling the
      linker what memory portion to put it in. We labeled our RAM, ram, with the constraints
      that it is writeable, allocatable, and executable. The linker will make sure with this
      that we can do all of those things.

      >ram   - This just tells the linker script to put this entire section (.text) into the
               ram region of memory. To my knowledge, the '>' does not mean "greater than". Instead,
               it is a symbol to let the linker know we want to put this in ram.

      AT>ram - This sets the LMA (load memory address) region to the same thing. LMA is the final
               translation of a VMA (virtual memory address). With this linker script, we're loading
               everything into its physical location. We'll let the kernel copy and sort out the
               virtual memory. That's why >ram and AT>ram are continually the same thing.

      :text  - This tells the linker script to put this into the :text program header. We've only
               defined three: text, data, and bss. In this case, we're telling the linker script
               to go into the text section.
    */
  } >ram AT>ram :text

  /*
   The global pointer allows the linker to position global variables and constants into
   independent positions relative to the gp (global pointer) register. The globals start
   after the text sections and are only relevant to the rodata, data, and bss sections.
  */
  PROVIDE(_global_pointer = .);

  /*
    Most compilers create a rodata (read only data) section for global constants. However,
    we're going to place ours in the text section. We can actually put this in :data, but
    since the .text section is read-only, we can place it there.

    NOTE: This doesn't actually do anything, yet. The actual "protection" cannot be done
    at link time. Instead, when we program the memory management unit (MMU), we will be
    able to choose which bits (R=read, W=write, X=execute) we want each memory segment
    to be able to do.
  */
  .rodata : {
    PROVIDE(_rodata_start = .);
    *(.rodata .rodata.*)
    PROVIDE(_rodata_end = .);

    /*
      Again, we're placing the rodata section in the memory segment "ram" and we're putting
      it in the :text program header. We don't have one for rodata anyway.
    */
  } >ram AT>ram :text

  .data : {

    /*
      . = ALIGN(4096) tells the linker to align the current memory location (which is
      0x8000_0000 + text section + rodata section) to 4096 bytes. This is because our paging
      system's resolution is 4,096 bytes or 4 KiB.
    */
    . = ALIGN(4096);
    PROVIDE(_data_start = .);

    /*
      sdata and data are essentially the same thing. However, compilers usually use the
      sdata sections for shorter, quicker loading sections. So, usually critical data
      is loaded there. However, we're loading all of this in one fell swoop.
      So, we're looking to put all of the following sections under the umbrella .data:
      .sdata
      .sdata.[anything]
      .data
      .data.[anything]

      ...in that order.
    */
    *(.sdata .sdata.*) *(.data .data.*)
    PROVIDE(_data_end = .);
  } >ram AT>ram :data

  .bss : {
    PROVIDE(_bss_start = .);
    *(.sbss .sbss.*) *(.bss .bss.*)
    PROVIDE(_bss_end = .);
  } >ram AT>ram :bss

  /*
    The following will be helpful when we allocate the kernel stack (_stack) and
    determine where the heap begnis and ends (_heap_start and _heap_start + _heap_size)/
    When we do memory allocation, we can use these symbols.

    We use the symbols instead of hard-coding an address because this is a floating target.
    As we add code, the heap moves farther down the memory and gets shorter.

    _memory_start will be set to 0x8000_0000 here. We use ORIGIN(ram) so that it will take
    whatever we set the origin of ram to. Otherwise, we'd have to change it more than once
    if we ever stray away from 0x8000_0000 as our entry point.
  */
  PROVIDE(_memory_start = ORIGIN(ram));
  /*
    Our kernel stack starts at the end of the bss segment (_bss_end). However, we're allocating
    0x80000 bytes (524 KiB) to our kernel stack. This should be PLENTY of space. The reason
    we add the memory is because the stack grows from higher memory to lower memory (bottom to top).
    Therefore we set the stack at the very bottom of its allocated slot.
    When we go to allocate from the stack, we'll subtract the number of bytes we need.
  */
  PROVIDE(_stack_start = _bss_end);
  PROVIDE(_stack_end = _stack_start + 0x80000);
  PROVIDE(_memory_end = ORIGIN(ram) + LENGTH(ram));

  /*
    Finally, our heap starts right after the kernel stack. This heap will be used mainly
    to dole out memory for user-space applications. However, in some circumstances, it will
    be used for kernel memory as well.

    We don't align here because we let the kernel determine how it wants to do this.
  */
  PROVIDE(_heap_start = _stack_end);
  PROVIDE(_heap_size = _memory_end - _heap_start);
}
