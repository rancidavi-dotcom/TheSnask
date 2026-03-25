package main

import (
	"fmt"
	"io/fs"
	"os"
	"path/filepath"
	"strconv"
)

func main() {
	if len(os.Args) < 3 {
		fmt.Fprintf(os.Stderr, "usage: %s <dir> <n>\n", os.Args[0])
		os.Exit(2)
	}
	dir := os.Args[1]
	n, _ := strconv.Atoi(os.Args[2])
	if n <= 0 {
		os.Exit(2)
	}
	_ = os.MkdirAll(dir, 0o755)

	buf := make([]byte, 1024)
	for i := range buf {
		buf[i] = 'a'
	}

	// create
	for i := 0; i < n; i++ {
		p := filepath.Join(dir, fmt.Sprintf("f_%06d.bin", i))
		if err := os.WriteFile(p, buf, 0o644); err != nil {
			panic(err)
		}
	}

	// list
	entries, err := os.ReadDir(dir)
	if err != nil {
		panic(err)
	}
	count := 0
	for _, e := range entries {
		if e.Name() == "." || e.Name() == ".." {
			continue
		}
		if e.Type().IsRegular() || (e.Type()&fs.ModeType) == 0 {
			count++
		} else {
			count++
		}
	}

	// delete
	for i := 0; i < n; i++ {
		p := filepath.Join(dir, fmt.Sprintf("f_%06d.bin", i))
		_ = os.Remove(p)
	}

	fmt.Println(count)
}

