package physical_memory

import (
	"errors"
	"simpledb/memory/iallocator"
	"simpledb/memory/pointer"
	"sync"
)

type memory struct {
	m      map[pointer.Pointer][]byte
	cnt    uint32
	locker sync.RWMutex
}

func New() iallocator.Allocator {
	return &memory{
		m:   map[pointer.Pointer][]byte{},
		cnt: 8,
	}
}

func (m *memory) Alloc(size uint32) (pointer.Pointer, error) {
	p := pointer.Pointer{
		BlockId:     0,
		BlockOffset: m.cnt,
	}
	m.cnt++
	m.m[p] = make([]byte, size)
	return p, nil
}

func (m *memory) Read(ptr pointer.Pointer) ([]byte, error) {
	data, ok := m.m[ptr]
	if ok {
		return data, nil
	} else {
		return nil, errors.New("invalid pointer " + ptr.String())
	}
}

func (m *memory) Write(ptr pointer.Pointer, offset uint32, data []byte) error {
	copy(m.m[ptr][offset:], data)
	return nil
}

func (m *memory) String() string {
	return "physical_memory"
}
