# lab4实验报告

## 完成内容
本次实验实现了 easy-fs 的硬链接（hard link）支持，并补齐了三个系统调用：

- **easy-fs 层**：给 `DiskInode` 增加 `nlink` 字段（记录指向该 inode 的硬链接数量，`initialize` 时置 1）,并且使直接数据段的数目减1个以达到128字节对齐，避免浪费磁盘空间；给 `Inode` 增加 `inode_id` 字段及 `inode_id()` 取值方法,便于硬链接时直接填入inode_id,而不是根据block_id和block_offset再去进行计算；实现 `Inode::link`，在根目录新增一个指向目标 inode 的目录项，并使目标 `nlink += 1`与 `Inode::unlink`删除目录项、`nlink -= 1`，当 `nlink == 0` 时回收数据块并释放 inode；在 `EasyFileSystem` 中补充 `dealloc_inode`，在删除硬链接的时候，如果目标inode的n_link为就回收。
- **os 层**：为 `File` trait 增加 `stat` 方法；定义 `Stat` 结构体（`dev`/`ino`/`mode`/`nlink`/`pad`，`#[repr(C)]`）及 `Stat::new`、`Stat::inode_stat` 构造器；为 `OSInode`、`Stdin`、`Stdout` 实现 `stat`。并实现相关系统调用。


## 问答题

### 在我们的 easy-fs 中，root inode 起着什么作用？如果 root inode 中的内容损坏了，会发生什么？


root inode 是整个文件系统的入口

它是文件系统格式化时分配的第一个 inode，inode_id 固定为 0，类型为 `Directory`。
easy-fs 是没有多层目录树——所有文件都以 `DirEntry` 的形式，直接存放在 root inode 的数据块里。也就是说，root inode 的 `direct`/`indirect1`/`indirect2` 所指向的数据块，维护着整个文件系统的映射表。

内核通过 `ROOT_INODE`完成所有文件操作,所有路径查找都必须从 root inode 出发。

如果 root inode 中的内容损坏了，

1. **无法启动内核**：内核启动时 `INITPROC` 会 `open_file`，从 root inode 里查找该目录项。如果查找失败，`open_file` 返回 `None`，紧接着的 `.unwrap()` 会触发 panic，操作系统无法启动。

2. **所有文件逻辑上丢失**：文件的查找是从`ROOT_INODE`开始的，如果他损坏了，也就没有办法索引文件了


# 荣誉准则
1.在完成本次实验的过程（含此前学习的过程）中，我曾分别与 以下各位 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：

豆包，deepseek

2.此外，我也参考了 以下资料 ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：

无特殊资料

3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。
