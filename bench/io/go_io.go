package main

import (
	"bufio"
	"fmt"
	"os"
	"strconv"
)

func main() {
	if len(os.Args) < 3 {
		fmt.Fprintf(os.Stderr, "usage: %s <path> <size_mb>\n", os.Args[0])
		os.Exit(2)
	}
	path := os.Args[1]
	sizeMB, _ := strconv.Atoi(os.Args[2])
	if sizeMB <= 0 {
		os.Exit(2)
	}

	chunk := make([]byte, 1024*1024)
	for i := range chunk {
		chunk[i] = 'a'
	}

	f, err := os.Create(path)
	if err != nil {
		panic(err)
	}
	w := bufio.NewWriterSize(f, len(chunk))
	for i := 0; i < sizeMB; i++ {
		if _, err := w.Write(chunk); err != nil {
			panic(err)
		}
	}
	w.Flush()
	f.Close()

	fi, err := os.Stat(path)
	if err != nil {
		panic(err)
	}
	fmt.Println(fi.Size())
}
