#ifndef RT_SFS_H
#define RT_SFS_H

#include "rt_base.h"

void sfs_read(SnaskValue* out, SnaskValue* path);
void sfs_write(SnaskValue* out, SnaskValue* path, SnaskValue* content);
void sfs_append(SnaskValue* out, SnaskValue* path, SnaskValue* content);
void sfs_write_mb(SnaskValue* out, SnaskValue* path, SnaskValue* mb_val);
void sfs_count_bytes(SnaskValue* out, SnaskValue* path);
void sfs_copy(SnaskValue* out, SnaskValue* src, SnaskValue* dst);
void sfs_move(SnaskValue* out, SnaskValue* src, SnaskValue* dst);
void sfs_mkdir(SnaskValue* out, SnaskValue* path);
void sfs_rmdir(SnaskValue* out, SnaskValue* path);
void sfs_is_file(SnaskValue* out, SnaskValue* path);
void sfs_is_dir(SnaskValue* out, SnaskValue* path);
void sfs_exists(SnaskValue* out, SnaskValue* path);
void sfs_delete(SnaskValue* out, SnaskValue* path);
void sfs_size(SnaskValue* out, SnaskValue* path);
void sfs_mtime(SnaskValue* out, SnaskValue* path);
void sfs_listdir(SnaskValue* out, SnaskValue* path);

// Benchmarking functions
void sfs_bench_create_small_files(SnaskValue* out, SnaskValue* dir, SnaskValue* n_files, SnaskValue* size_bytes);
void sfs_bench_count_entries(SnaskValue* out, SnaskValue* dir);
void sfs_bench_delete_small_files(SnaskValue* out, SnaskValue* dir, SnaskValue* n_files);

#endif // RT_SFS_H
