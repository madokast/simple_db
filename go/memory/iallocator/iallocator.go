package iallocator

import "simpledb/memory/pointer"

type Allocator interface {
	Alloc(size uint32) (pointer.Pointer, error)                  // 分配内存
	Read(ptr pointer.Pointer) ([]byte, error)                    // 读取
	Write(ptr pointer.Pointer, offset uint32, data []byte) error // 写入
	String() string
}
