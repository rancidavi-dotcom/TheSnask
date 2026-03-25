#pragma once

#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

// Returns "skia-backend" or a more specific version string.
const char* snask_skia_impl_version(void);

// Surfaces (offscreen raster)
int snask_skia_impl_surface_create(int w, int h);
int snask_skia_impl_surface_width(int id);
int snask_skia_impl_surface_height(int id);

bool snask_skia_impl_surface_clear(int id, double r, double g, double b, double a);
bool snask_skia_impl_surface_set_color(int id, double r, double g, double b, double a);

bool snask_skia_impl_draw_rect(int id, double x, double y, double w, double h, bool fill);
bool snask_skia_impl_draw_circle(int id, double cx, double cy, double radius, bool fill);
bool snask_skia_impl_draw_line(int id, double x1, double y1, double x2, double y2, double stroke_w);
bool snask_skia_impl_draw_text(int id, double x, double y, const char* text, double size);

bool snask_skia_impl_save_png(int id, const char* path);

#ifdef __cplusplus
}
#endif

