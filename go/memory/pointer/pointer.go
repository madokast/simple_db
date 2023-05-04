package pointer

import (
	"simpledb/memory/memutils"
	"strconv"
	"strings"
	"unsafe"
)

// Pointer 指针
type Pointer struct {
	BlockId     uint32 // 可以看作文件编号
	BlockOffset uint32 // 可以看作文件内偏移。blockId 和 blockOffset 一起完成寻址
}

const Size = uint32(unsafe.Sizeof(Pointer{}))

var NullPointer = Pointer{}

func (p *Pointer) IsNull() bool {
	return p.BlockId == 0 && p.BlockOffset == 0
}

func (p *Pointer) ToBytes() []byte {
	bytes := make([]byte, Size)
	memutils.MemWrite(p, Size, bytes)
	return bytes
}

func (p *Pointer) String() string {
	if p.IsNull() {
		return "nullptr"
	}
	id := strings.ToUpper(strconv.FormatInt(int64(p.BlockId), 16))
	offset := strings.ToUpper(strconv.FormatInt(int64(p.BlockOffset), 16))

	if id == "0" {
		return offset
	}

	return id + "X" + offset
}
