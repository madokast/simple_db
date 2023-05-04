package physical_memory

import (
	"testing"
)

func TestNew(t *testing.T) {
	allocator := New()
	p, err := allocator.Alloc(5)
	if err != nil {
		panic(err)
	}
	err = allocator.Write(p, 0, []byte("hello"))
	if err != nil {
		panic(err)
	}
	bytes, err := allocator.Read(p)
	if err != nil {
		panic(err)
	}
	t.Log(string(bytes))
}
