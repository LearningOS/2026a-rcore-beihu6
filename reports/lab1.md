# ch3实验报告
## 完成内容
为每一个线程维护一个 SYSCALL_COUNT 数组，当触发系统调用时，根据 syscall_id 将对应元素加 1，以实现记录系统调用次数。

## 问答题
### 实验环境
- 版本：0.4.1
### 1. 三个 bad 测例的测试与现象
我实际运行了 `make -C ci-user test CHAPTER=2`，并在日志中观察到以下现象：

#### (1) 空指针访问：`ch2b_bad_address.rs`
程序功能：执行 `*(0x0 as *mut u8) = 0;`，对地址 0x0 进行写入。

程序错误现象：
> `[kernel] PageFault in application, bad addr = 0x0, bad instruction = 0x804003a4, kernel killed it.`

说明：
- 用户态访问了无效地址 0x0；
- 地址不在合法页表映射范围内；
- CPU 触发了 StorePageFault/PageFault；
- 内核捕获后终止了该应用。

#### (2) 使用 S 态特权指令：`ch2b_bad_instructions.rs`
程序功能：在 U 态中执行 `sret`，这是 S 态特权返回指令。

程序错误现象：
> `[kernel] IllegalInstruction in application, kernel killed it.`

说明：
- `sret` 只能在 S 模式或更高权限下安全使用；
- 在 U 模式执行该指令属于特权越权；
- CPU 直接抛出 IllegalInstruction 异常，内核杀死应用。

#### (3) 访问 S 态寄存器：`ch2b_bad_register.rs`
程序功能：执行 `csrr rd, sstatus`，读取 S 态寄存器 `sstatus`。

程序错误现象：
> `[kernel] IllegalInstruction in application, kernel killed it.`

说明：
- `sstatus` 是 S 模式控制状态寄存器；
- U 模式不能直接访问它；
- 读取该 CSR 会触发 IllegalInstruction，内核终止程序。

### 2.
#### (1)刚进入 __restore 时，sp 代表了什么值。请指出 __restore 的两种使用情景。
刚进入时，sp代表着用户栈顶地址。


__restore有两种使用情景：
* 处理trap之后恢复上下文
* 用于新任务的启动，新任务将上下文写入内核栈，并设计entry,内核执行__restore,经sret跳转到entry

##### (2)这几行汇编代码特殊处理了哪些寄存器？这些寄存器的的值对于进入用户态有何意义？请分别解释。
```c
ld t0, 32*8(sp)
ld t1, 33*8(sp)
ld t2, 2*8(sp)
csrw sstatus, t0
csrw sepc, t1
csrw sscratch, t2
```
将进入trap时保存的sepc和sstatus还原，同时将sscratch设置为用户栈指针，用于之后与sp交换

意义：
* 还原trap时的状态，处理trap时可能出现嵌套trap,这样可能会修改sstatus与sepc
* 将sp设置为正确的用户栈栈顶地址

#### (3)为何跳过了 x2 和 x4？
```c
ld x1, 1*8(sp)
ld x3, 3*8(sp)
.set n, 5
.rept 27
   LOAD_GP %n
   .set n, n+1
.endr
```
* x4:在riscv中，x4一般不拿来使用，在保存上下文的时候也没有保存x4,栈中是垃圾值，所以不能读取内存值给x4
* x2(sp):当前上下文保存在内核栈中，sp要指向它，用于恢复上下文，一旦修改，sp就会指向用户栈，导致上下文恢复失败，sp会在恢复上下文后，通过csrrw sp,sscratch,sp修改

#### (4)L60：该指令之后，sp 和 sscratch 中的值分别有什么意义？
```c
csrrw sp,sscratch,sp
```
* sp->user stack,用于用户程序执行
* sscratch->kernel stack,内核栈，用于之后trap处理

#### (5)__restore：中发生状态切换在哪一条指令？为何该指令执行之后会进入用户态？
发生在`sret`。

`sret` 会从 `sstatus` 寄存器的 **SPP 位（Previous Privilege Level）** 读取上一次的特权级；
在我们的任务恢复流程里，陷入 S 态之前任务是 U 态，所以 `SPP=0`。
执行`sret`时：

1. 把 `sepc` 的值写入 PC，回到用户程序地址；
2. 将特权级设置为 `sstatus.SPP` 保存的值（0 = User 用户态）；
3. 同时自动把 `SPP` 置 1（S 态），为下一次陷入做好标记。

#### (6)L13：该指令之后，sp 和 sscratch 中的值分别有什么意义？
```c
csrrw sp,sscratch,sp
```
* sp->kernel stack,用于上下文保存
* sscratch->user stack,用于中转用户sp,保存到上下文中

#### (7)从 U 态进入 S 态是哪一条指令发生的？
`ecall`   

# 荣誉准则
1.在完成本次实验的过程（含此前学习的过程）中，我曾分别与 以下各位 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：

* deepseek,豆包，gpt 5.3

2.此外，我也参考了 以下资料 ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：

* 无特殊资料，不懂得询问了ai

3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。