#include "skia_bridge.h"

#include <vector>
#include <string>

#include "include/core/SkCanvas.h"
#include "include/core/SkColor.h"
#include "include/core/SkFont.h"
#include "include/core/SkImageInfo.h"
#include "include/core/SkPaint.h"
#include "include/core/SkRect.h"
#include "include/core/SkSurface.h"
#include "include/core/SkTextBlob.h"
#include "include/core/SkTypes.h"
#include "include/utils/SkTextUtils.h"
#include "include/encode/SkPngEncoder.h"
#include "include/core/SkStream.h"

struct SnaskSkiaSurface {
    int w = 0;
    int h = 0;
    float r = 1.0f;
    float g = 1.0f;
    float b = 1.0f;
    float a = 1.0f;
    sk_sp<SkSurface> surface;
};

static std::vector<SnaskSkiaSurface> g_surfaces;

static SnaskSkiaSurface* get_surface(int id) {
    if (id < 0) return nullptr;
    size_t idx = static_cast<size_t>(id);
    if (idx >= g_surfaces.size()) return nullptr;
    if (!g_surfaces[idx].surface) return nullptr;
    return &g_surfaces[idx];
}

const char* snask_skia_impl_version(void) {
    return "skia-backend";
}

int snask_skia_impl_surface_create(int w, int h) {
    if (w <= 0 || h <= 0 || w > 16384 || h > 16384) return -1;
    SkImageInfo info = SkImageInfo::MakeN32Premul(w, h);
    sk_sp<SkSurface> surface = SkSurfaces::Raster(info);
    if (!surface) return -1;

    SnaskSkiaSurface s;
    s.w = w;
    s.h = h;
    s.surface = surface;

    // Default clear transparent.
    s.surface->getCanvas()->clear(SK_ColorTRANSPARENT);

    g_surfaces.push_back(std::move(s));
    return static_cast<int>(g_surfaces.size() - 1);
}

int snask_skia_impl_surface_width(int id) {
    auto* s = get_surface(id);
    return s ? s->w : -1;
}

int snask_skia_impl_surface_height(int id) {
    auto* s = get_surface(id);
    return s ? s->h : -1;
}

bool snask_skia_impl_surface_clear(int id, double r, double g, double b, double a) {
    auto* s = get_surface(id);
    if (!s) return false;
    SkColor4f c = SkColor4f{(float)r, (float)g, (float)b, (float)a};
    s->surface->getCanvas()->clear(c.toSkColor());
    return true;
}

bool snask_skia_impl_surface_set_color(int id, double r, double g, double b, double a) {
    auto* s = get_surface(id);
    if (!s) return false;
    s->r = (float)r;
    s->g = (float)g;
    s->b = (float)b;
    s->a = (float)a;
    return true;
}

static SkPaint make_paint(const SnaskSkiaSurface& s) {
    SkPaint p;
    p.setAntiAlias(true);
    p.setColor4f(SkColor4f{s.r, s.g, s.b, s.a});
    return p;
}

bool snask_skia_impl_draw_rect(int id, double x, double y, double w, double h, bool fill) {
    auto* s = get_surface(id);
    if (!s) return false;
    SkPaint p = make_paint(*s);
    p.setStyle(fill ? SkPaint::kFill_Style : SkPaint::kStroke_Style);
    s->surface->getCanvas()->drawRect(SkRect::MakeXYWH((float)x, (float)y, (float)w, (float)h), p);
    return true;
}

bool snask_skia_impl_draw_circle(int id, double cx, double cy, double radius, bool fill) {
    auto* s = get_surface(id);
    if (!s) return false;
    SkPaint p = make_paint(*s);
    p.setStyle(fill ? SkPaint::kFill_Style : SkPaint::kStroke_Style);
    s->surface->getCanvas()->drawCircle((float)cx, (float)cy, (float)radius, p);
    return true;
}

bool snask_skia_impl_draw_line(int id, double x1, double y1, double x2, double y2, double stroke_w) {
    auto* s = get_surface(id);
    if (!s) return false;
    SkPaint p = make_paint(*s);
    p.setStyle(SkPaint::kStroke_Style);
    p.setStrokeWidth(stroke_w > 0.0 ? (float)stroke_w : 1.0f);
    s->surface->getCanvas()->drawLine((float)x1, (float)y1, (float)x2, (float)y2, p);
    return true;
}

bool snask_skia_impl_draw_text(int id, double x, double y, const char* text, double size) {
    auto* s = get_surface(id);
    if (!s || !text) return false;
    SkPaint p = make_paint(*s);
    SkFont font;
    font.setSize(size > 0.0 ? (float)size : 14.0f);
    SkTextUtils::DrawString(s->surface->getCanvas(), text, (float)x, (float)y, font, p, SkTextUtils::kLeft_Align);
    return true;
}

bool snask_skia_impl_save_png(int id, const char* path) {
    auto* s = get_surface(id);
    if (!s || !path) return false;

    SkPixmap pixmap;
    if (!s->surface->peekPixels(&pixmap)) return false;

    SkFILEWStream stream(path);
    if (!stream.isValid()) return false;

    SkPngEncoder::Options opts;
    opts.fZLibLevel = 6;
    return SkPngEncoder::Encode(&stream, pixmap, opts);
}

