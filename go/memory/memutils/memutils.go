package memutils

import (
	"reflect"
	"unsafe"
)

// AsBytes 注意内存泄露
func AsBytes[T interface{}](t *T, size uint32) []byte {
	return *(*[]byte)(unsafe.Pointer(&reflect.SliceHeader{
		Data: uintptr(unsafe.Pointer(t)),
		Cap:  int(size),
		Len:  int(size),
	}))
}

// MemWrite 将指针 t 指向的 size 区域的内存复制到 mem，返回剩余内存
func MemWrite[T interface{}](t *T, size uint32, mem []byte) []byte {
	src := AsBytes(t, size)
	copy(mem, src)
	return mem[size:]
}
