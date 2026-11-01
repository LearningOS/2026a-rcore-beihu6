# lab5实验报告

## 完成内容

在 ch8 框架上实现了基于银行家算法的死锁检测，实现`sys_get_time`。

- **进程控制块**：在 `ProcessControlBlockInner` 中新增 `deadlock_detect` 开关，以及四张按 `[tid][资源 id]` 索引的矩阵：`mutex_allocation`、`mutex_need`、`semaphore_allocation`、`semaphore_need`。前两张分别记录每个线程当前持有的、正在等待的 mutex 数量，后两张对应 semaphore。
- **aviable矩阵**：mutex 是二值资源，容量恒为 1；`Semaphore` 新增 `total` 字段保存创建时的资源总数，用于计算 `Available = total - ΣAllocation`。

## 实现原理

### 数据结构

把每个线程对每类资源的持有量和正在等待的数量分别记录成 Allocation 与 Need 矩阵。线程发起 `mutex_lock` / `semaphore_down` 时：

1. 先允许分配，把本次请求累加到该线程的 Need 中；
2. 以 `Work = Available = total - ΣAllocation` 为初始工作向量；
3. 反复寻找满足 `Need[i] <= Work` 的线程，让其“运行完成”并把 `Allocation[i]` 归还给 Work；
4. 若所有线程都能完成，则状态安全，允许继续获取资源；否则撤销本次 Need 并返回 `-0xDEAD`。

线程真正拿到资源后，把对应数量从 Need 移到 Allocation；`unlock` / `up` 时再减少 Allocation。


# 问答题
### 在我们的多线程实现中，当主线程 (即 0 号线程) 退出时，视为整个进程退出， 此时需要结束该进程管理的所有线程并回收其资源。 - 需要回收的资源有哪些？ - 其他线程的 TaskControlBlock 可能在哪些位置被引用，分别是否需要回收，为什么？
* 需要回收的资源：memory_set,fd_table,信号量，锁，条件变量资源，线程资源，tasks.clear,减少TCB引用计数，子进程归init_proc,回收children_process,内核栈暂不释放，add_stopping_task将会对主线程的TCB加一个强引用计数，延迟回收
* TCB会被引用的地方以及是否要回收：PCB内部的tasks,manager的就绪队列，全局 current_task以及同步资源的相关队列
* `take_current_task()`把当前任务取出来，current_task 置空，如果是主线程，延迟回收，否则直接drop。PCB内部的引用要清除，就绪队列的引用也要清除，代码中遍历 tasks，调用`remove_inactive_task`，最后`process_inner.tasks.clear()`实现清除，同步资源的引用虽PCB被drop实现清除。

### 对比以下两种 Mutex 中的实现，二者有什么区别？这些区别可能会导致什么问题？
* mutex1:使用 `loop`，线程被唤醒后循环重新检查锁状态。
* mutex2：只用一次 `if`，唤醒之后不再检查锁状态。

问题：mutex1正确，mutex2会在线程被唤醒后，没有检查锁的状态，线程误认为拿到了锁

* mutex1：unlock 中先将 locked 置 false，再唤醒等待队列里的线程。
* mutex2：unlock 优先唤醒线程,仅当等待队列为空时，才把 locked 置 false；队列非空时永远不释放锁标记。

问题：mutex1正确，mutex2只要有线程在等待，locked一直为true,又被阻塞等待，locked永远为true,造成死锁

# 荣誉准则
1.在完成本次实验的过程（含此前学习的过程）中，我曾分别与 以下各位 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：

豆包，deepseek

2.此外，我也参考了 以下资料 ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：

无特殊资料

3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。
