#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <math.h>
#include <dlfcn.h>
#include "rt_gui.h"
#include "rt_gc.h"

#ifdef SNASK_GUI_GTK
#include <gtk/gtk.h>
#include <cairo.h>

static char* gui_ptr_to_handle(void* p) {
    char buf[64];
    snprintf(buf, sizeof(buf), "%p", p);
    return snask_gc_strdup(buf);
}

static void* gui_handle_to_ptr(const char* h) {
    if (!h) return NULL;
    void* p = NULL;
    sscanf(h, "%p", &p);
    return p;
}

typedef struct {
    GtkWidget* window;
    GtkWidget* area;
    unsigned char* pixels;
    int width;
    int height;
    int scale;
    int should_close;
    unsigned char keys[512];
    double mouse_x;
    double mouse_y;
    int mouse_btns[5];
} SnaskGuiWindow;

static gboolean snaskgui_draw_cb(GtkWidget* widget, cairo_t* cr, gpointer user_data) {
    (void)widget;
    SnaskGuiWindow* win = (SnaskGuiWindow*)user_data;
    if (!win || !win->pixels || win->width <= 0 || win->height <= 0) return FALSE;

    cairo_surface_t* surface = cairo_image_surface_create_for_data(
        win->pixels,
        CAIRO_FORMAT_ARGB32,
        win->width,
        win->height,
        win->width * 4
    );
    cairo_save(cr);
    cairo_scale(cr, (double)win->scale, (double)win->scale);
    cairo_set_source_surface(cr, surface, 0, 0);
    cairo_pattern_set_filter(cairo_get_source(cr), CAIRO_FILTER_NEAREST);
    cairo_paint(cr);
    cairo_restore(cr);
    cairo_surface_destroy(surface);
    return FALSE;
}

static int snaskgui_key_index(guint keyval) {
    switch (keyval) {
        case GDK_KEY_Up: return 1;
        case GDK_KEY_Down: return 2;
        case GDK_KEY_Left: return 3;
        case GDK_KEY_Right: return 4;
        case GDK_KEY_z:
        case GDK_KEY_Z: return 5;
        case GDK_KEY_x:
        case GDK_KEY_X: return 6;
        case GDK_KEY_Return: return 7;
        case GDK_KEY_Shift_L:
        case GDK_KEY_Shift_R: return 8;
        case GDK_KEY_Escape: return 9;
        default: break;
    }
    if (keyval < 512) return (int)keyval;
    return 0;
}

static gboolean snaskgui_key_press_cb(GtkWidget* widget, GdkEventKey* event, gpointer user_data) {
    (void)widget;
    SnaskGuiWindow* win = (SnaskGuiWindow*)user_data;
    if (!win || !event) return FALSE;
    int idx = snaskgui_key_index(event->keyval);
    if (idx >= 0 && idx < 512) win->keys[idx] = 1;
    if (idx == 9) win->should_close = 1;
    return FALSE;
}

static gboolean snaskgui_key_release_cb(GtkWidget* widget, GdkEventKey* event, gpointer user_data) {
    (void)widget;
    SnaskGuiWindow* win = (SnaskGuiWindow*)user_data;
    if (!win || !event) return FALSE;
    int idx = snaskgui_key_index(event->keyval);
    if (idx >= 0 && idx < 512) win->keys[idx] = 0;
    return FALSE;
}

static gboolean snaskgui_motion_cb(GtkWidget* widget, GdkEventMotion* event, gpointer user_data) {
    (void)widget;
    SnaskGuiWindow* win = (SnaskGuiWindow*)user_data;
    if (win && event) {
        win->mouse_x = event->x / win->scale;
        win->mouse_y = event->y / win->scale;
    }
    return FALSE;
}

static gboolean snaskgui_button_cb(GtkWidget* widget, GdkEventButton* event, gpointer user_data) {
    (void)widget;
    SnaskGuiWindow* win = (SnaskGuiWindow*)user_data;
    if (win && event) {
        if (event->button < 5) {
            if (event->type == GDK_BUTTON_PRESS) win->mouse_btns[event->button] = 1;
            else if (event->type == GDK_BUTTON_RELEASE) win->mouse_btns[event->button] = 0;
        }
    }
    return FALSE;
}

static void snaskgui_destroy_cb(GtkWidget* widget, gpointer user_data) {
    (void)widget;
    SnaskGuiWindow* win = (SnaskGuiWindow*)user_data;
    if (win) win->should_close = 1;
}

static SnaskGuiWindow* snaskgui_from_handle(SnaskValue* h) {
    if (!h || (int)h->tag != SNASK_STR) return NULL;
    return (SnaskGuiWindow*)gui_handle_to_ptr((const char*)h->ptr);
}

typedef struct {
    char* handler_name;
    char* widget_handle;
    char* ctx;
} GuiCallbackCtx;

static void gui_free_ctx(GuiCallbackCtx* ctx) {
    if (!ctx) return;
    if (ctx->handler_name) free(ctx->handler_name);
    if (ctx->widget_handle) free(ctx->widget_handle);
    if (ctx->ctx) free(ctx->ctx);
    free(ctx);
}

static SnaskValue gui_call_handler_1(const char* handler_name, const char* widget_handle) {
    if (!handler_name) return MAKE_NIL();
    char sym[512];
    snprintf(sym, sizeof(sym), "f_%s", handler_name);
    void* fp = dlsym(NULL, sym);
    if (!fp) return MAKE_NIL();

    typedef void (*SnaskFn1)(SnaskValue* ra, SnaskValue* a1);
    SnaskFn1 f = (SnaskFn1)fp;

    SnaskValue ra = MAKE_NIL();
    SnaskValue wh = MAKE_STR(snask_gc_strdup(widget_handle ? widget_handle : ""));
    f(&ra, &wh);
    return ra;
}

static SnaskValue gui_call_handler_2(const char* handler_name, const char* widget_handle, const char* ctx) {
    if (!handler_name) return MAKE_NIL();
    char sym[512];
    snprintf(sym, sizeof(sym), "f_%s", handler_name);
    void* fp = dlsym(NULL, sym);
    if (!fp) return MAKE_NIL();

    typedef void (*SnaskFn2)(SnaskValue* ra, SnaskValue* a1, SnaskValue* a2);
    SnaskFn2 f = (SnaskFn2)fp;

    SnaskValue ra = MAKE_NIL();
    SnaskValue wh = MAKE_STR(snask_gc_strdup(widget_handle ? widget_handle : ""));
    SnaskValue cv = MAKE_STR(snask_gc_strdup(ctx ? ctx : ""));
    f(&ra, &wh, &cv);
    return ra;
}

static void gui_on_button_clicked(GtkWidget* _widget, gpointer user_data) {
    (void)_widget;
    GuiCallbackCtx* ctx = (GuiCallbackCtx*)user_data;
    if (!ctx) return;
    if (ctx->ctx) (void)gui_call_handler_2(ctx->handler_name, ctx->widget_handle, ctx->ctx);
    else (void)gui_call_handler_1(ctx->handler_name, ctx->widget_handle);
}

void gui_init(SnaskValue* out) {
    int argc = 0;
    char** argv = NULL;
    gboolean ok = gtk_init_check(&argc, &argv);
    *out = MAKE_BOOL(ok);
}

void gui_quit(SnaskValue* out) {
    gtk_main_quit();
    *out = MAKE_NIL();
}

void gui_run(SnaskValue* out) {
    gtk_main();
    *out = MAKE_NIL();
}

void gui_window(SnaskValue* out, SnaskValue* title, SnaskValue* w, SnaskValue* h) {
    if ((int)title->tag != SNASK_STR || (int)w->tag != SNASK_NUM || (int)h->tag != SNASK_NUM) { *out = MAKE_NIL(); return; }
    GtkWidget* win = gtk_window_new(GTK_WINDOW_TOPLEVEL);
    gtk_window_set_title(GTK_WINDOW(win), (const char*)title->ptr);
    gtk_window_set_default_size(GTK_WINDOW(win), (int)w->num, (int)h->num);
    g_signal_connect(win, "destroy", G_CALLBACK(gtk_main_quit), NULL);
    *out = MAKE_STR(gui_ptr_to_handle(win));
}

void gui_set_title(SnaskValue* out, SnaskValue* win_h, SnaskValue* title) {
    if ((int)win_h->tag != SNASK_STR || (int)title->tag != SNASK_STR) { *out = MAKE_NIL(); return; }
    GtkWidget* win = (GtkWidget*)gui_handle_to_ptr((const char*)win_h->ptr);
    if (!win || !GTK_IS_WINDOW(win)) { *out = MAKE_NIL(); return; }
    gtk_window_set_title(GTK_WINDOW(win), (const char*)title->ptr);
    *out = MAKE_BOOL(true);
}

void gui_set_resizable(SnaskValue* out, SnaskValue* win_h, SnaskValue* resizable) {
    if ((int)win_h->tag != SNASK_STR || (int)resizable->tag != SNASK_BOOL) { *out = MAKE_NIL(); return; }
    GtkWidget* win = (GtkWidget*)gui_handle_to_ptr((const char*)win_h->ptr);
    if (!win || !GTK_IS_WINDOW(win)) { *out = MAKE_NIL(); return; }
    gtk_window_set_resizable(GTK_WINDOW(win), resizable->num != 0.0);
    *out = MAKE_BOOL(true);
}

void gui_autosize(SnaskValue* out, SnaskValue* win_h) {
    if ((int)win_h->tag != SNASK_STR) { *out = MAKE_NIL(); return; }
    GtkWidget* win = (GtkWidget*)gui_handle_to_ptr((const char*)win_h->ptr);
    if (!win || !GTK_IS_WINDOW(win)) { *out = MAKE_NIL(); return; }
    gtk_window_resize(GTK_WINDOW(win), 1, 1);
    *out = MAKE_BOOL(true);
}

void gui_vbox(SnaskValue* out) {
    GtkWidget* box = gtk_box_new(GTK_ORIENTATION_VERTICAL, 8);
    *out = MAKE_STR(gui_ptr_to_handle(box));
}

void gui_hbox(SnaskValue* out) {
    GtkWidget* box = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 8);
    *out = MAKE_STR(gui_ptr_to_handle(box));
}

void gui_eventbox(SnaskValue* out) {
    GtkWidget* eb = gtk_event_box_new();
    *out = MAKE_STR(gui_ptr_to_handle(eb));
}

void gui_scrolled(SnaskValue* out) {
    GtkWidget* sw = gtk_scrolled_window_new(NULL, NULL);
    gtk_scrolled_window_set_policy(GTK_SCROLLED_WINDOW(sw), GTK_POLICY_AUTOMATIC, GTK_POLICY_AUTOMATIC);
    *out = MAKE_STR(gui_ptr_to_handle(sw));
}

void gui_flowbox(SnaskValue* out) {
    GtkWidget* fb = gtk_flow_box_new();
    *out = MAKE_STR(gui_ptr_to_handle(fb));
}

void gui_flow_add(SnaskValue* out, SnaskValue* flow_h, SnaskValue* child_h) {
    if ((int)flow_h->tag != SNASK_STR || (int)child_h->tag != SNASK_STR) { *out = MAKE_NIL(); return; }
    GtkWidget* flow = (GtkWidget*)gui_handle_to_ptr((const char*)flow_h->ptr);
    GtkWidget* child = (GtkWidget*)gui_handle_to_ptr((const char*)child_h->ptr);
    if (!flow || !child || !GTK_IS_FLOW_BOX(flow)) { *out = MAKE_NIL(); return; }
    gtk_flow_box_insert(GTK_FLOW_BOX(flow), child, -1);
    *out = MAKE_BOOL(true);
}

void gui_frame(SnaskValue* out) {
    GtkWidget* f = gtk_frame_new(NULL);
    *out = MAKE_STR(gui_ptr_to_handle(f));
}

void gui_set_margin(SnaskValue* out, SnaskValue* widget_h, SnaskValue* margin_v) {
    if ((int)widget_h->tag != SNASK_STR || (int)margin_v->tag != SNASK_NUM) { *out = MAKE_NIL(); return; }
    GtkWidget* w = (GtkWidget*)gui_handle_to_ptr((const char*)widget_h->ptr);
    if (!w) { *out = MAKE_NIL(); return; }
    int m = (int)margin_v->num;
    gtk_widget_set_margin_start(w, m);
    gtk_widget_set_margin_end(w, m);
    gtk_widget_set_margin_top(w, m);
    gtk_widget_set_margin_bottom(w, m);
    *out = MAKE_BOOL(true);
}

void gui_icon(SnaskValue* out, SnaskValue* name, SnaskValue* size_v) {
    if ((int)name->tag != SNASK_STR || (int)size_v->tag != SNASK_NUM) { *out = MAKE_NIL(); return; }
    GtkWidget* img = gtk_image_new_from_icon_name((const char*)name->ptr, GTK_ICON_SIZE_DIALOG);
    if (GTK_IS_IMAGE(img)) gtk_image_set_pixel_size(GTK_IMAGE(img), (int)size_v->num);
    *out = MAKE_STR(gui_ptr_to_handle(img));
}

void gui_css(SnaskValue* out, SnaskValue* css) {
    if ((int)css->tag != SNASK_STR) { *out = MAKE_NIL(); return; }
    GtkCssProvider* provider = gtk_css_provider_new();
    gtk_css_provider_load_from_data(provider, (const char*)css->ptr, -1, NULL);
    GdkScreen* screen = gdk_screen_get_default();
    if (screen) {
        gtk_style_context_add_provider_for_screen(
            screen,
            GTK_STYLE_PROVIDER(provider),
            GTK_STYLE_PROVIDER_PRIORITY_USER
        );
    }
    g_object_unref(provider);
    *out = MAKE_BOOL(true);
}

void gui_add_class(SnaskValue* out, SnaskValue* widget_h, SnaskValue* cls) {
    if ((int)widget_h->tag != SNASK_STR || (int)cls->tag != SNASK_STR) { *out = MAKE_NIL(); return; }
    GtkWidget* w = (GtkWidget*)gui_handle_to_ptr((const char*)widget_h->ptr);
    if (!w) { *out = MAKE_NIL(); return; }
    GtkStyleContext* sc = gtk_widget_get_style_context(w);
    if (sc) gtk_style_context_add_class(sc, (const char*)cls->ptr);
    *out = MAKE_BOOL(true);
}

static gboolean gui_on_tap_cb(GtkWidget* _widget, GdkEventButton* _ev, gpointer user_data) {
    (void)_widget; (void)_ev;
    GuiCallbackCtx* ctx = (GuiCallbackCtx*)user_data;
    if (!ctx) return FALSE;
    if (ctx->ctx) (void)gui_call_handler_2(ctx->handler_name, ctx->widget_handle, ctx->ctx);
    else (void)gui_call_handler_1(ctx->handler_name, ctx->widget_handle);
    return FALSE;
}

void gui_on_tap_ctx(SnaskValue* out, SnaskValue* widget_h, SnaskValue* handler_name, SnaskValue* ctx_str) {
    if ((int)widget_h->tag != SNASK_STR || (int)handler_name->tag != SNASK_STR || (int)ctx_str->tag != SNASK_STR) { *out = MAKE_NIL(); return; }
    GtkWidget* w = (GtkWidget*)gui_handle_to_ptr((const char*)widget_h->ptr);
    if (!w) { *out = MAKE_NIL(); return; }
    gtk_widget_add_events(w, GDK_BUTTON_PRESS_MASK);
    GuiCallbackCtx* ctx = (GuiCallbackCtx*)calloc(1, sizeof(GuiCallbackCtx));
    ctx->handler_name = strdup((const char*)handler_name->ptr);
    ctx->widget_handle = strdup((const char*)widget_h->ptr);
    ctx->ctx = strdup((const char*)ctx_str->ptr);
    g_signal_connect_data(w, "button-press-event", G_CALLBACK(gui_on_tap_cb), ctx, (GClosureNotify)gui_free_ctx, 0);
    *out = MAKE_BOOL(true);
}

void gui_listbox(SnaskValue* out) {
    GtkWidget* lb = gtk_list_box_new();
    gtk_list_box_set_selection_mode(GTK_LIST_BOX(lb), GTK_SELECTION_SINGLE);
    *out = MAKE_STR(gui_ptr_to_handle(lb));
}

void gui_list_add_text(SnaskValue* out, SnaskValue* list_h, SnaskValue* text) {
    if ((int)list_h->tag != SNASK_STR || (int)text->tag != SNASK_STR) { *out = MAKE_NIL(); return; }
    GtkWidget* w = (GtkWidget*)gui_handle_to_ptr((const char*)list_h->ptr);
    if (!w || !GTK_IS_LIST_BOX(w)) { *out = MAKE_NIL(); return; }

    GtkWidget* lbl = gtk_label_new((const char*)text->ptr);
    gtk_widget_set_halign(lbl, GTK_ALIGN_START);

    GtkWidget* row = gtk_list_box_row_new();
    gtk_container_add(GTK_CONTAINER(row), lbl);
    gtk_widget_show_all(row);
    gtk_list_box_insert(GTK_LIST_BOX(w), row, -1);
    g_object_set_data_full(G_OBJECT(row), "snask_pkg", strdup((const char*)text->ptr), free);

    *out = MAKE_STR(gui_ptr_to_handle(row));
}

static void gui_on_list_selected(GtkListBox* _box, GtkListBoxRow* row, gpointer user_data) {
    (void)_box;
    GuiCallbackCtx* ctx = (GuiCallbackCtx*)user_data;
    if (!ctx || !row) return;
    const char* pkg = (const char*)g_object_get_data(G_OBJECT(row), "snask_pkg");
    if (!pkg) pkg = "";
    if (ctx->ctx) (void)gui_call_handler_2(ctx->handler_name, pkg, ctx->ctx);
    else (void)gui_call_handler_1(ctx->handler_name, pkg);
}

void gui_on_select_ctx(SnaskValue* out, SnaskValue* list_h, SnaskValue* handler_name, SnaskValue* ctx_str) {
    if ((int)list_h->tag != SNASK_STR || (int)handler_name->tag != SNASK_STR || (int)ctx_str->tag != SNASK_STR) { *out = MAKE_NIL(); return; }
    GtkWidget* w = (GtkWidget*)gui_handle_to_ptr((const char*)list_h->ptr);
    if (!w || !GTK_IS_LIST_BOX(w)) { *out = MAKE_NIL(); return; }
    GuiCallbackCtx* ctx = (GuiCallbackCtx*)calloc(1, sizeof(GuiCallbackCtx));
    ctx->handler_name = strdup((const char*)handler_name->ptr);
    ctx->widget_handle = strdup((const char*)list_h->ptr);
    ctx->ctx = strdup((const char*)ctx_str->ptr);
    g_signal_connect_data(w, "row-selected", G_CALLBACK(gui_on_list_selected), ctx, (GClosureNotify)gui_free_ctx, 0);
    *out = MAKE_BOOL(true);
}

void gui_set_child(SnaskValue* out, SnaskValue* parent_h, SnaskValue* child_h) {
    if ((int)parent_h->tag != SNASK_STR || (int)child_h->tag != SNASK_STR) { *out = MAKE_NIL(); return; }
    GtkWidget* parent = (GtkWidget*)gui_handle_to_ptr((const char*)parent_h->ptr);
    GtkWidget* child = (GtkWidget*)gui_handle_to_ptr((const char*)child_h->ptr);
    if (!parent || !child) { *out = MAKE_NIL(); return; }
    if (GTK_IS_BIN(parent)) {
        GtkWidget* old = gtk_bin_get_child(GTK_BIN(parent));
        if (old) gtk_container_remove(GTK_CONTAINER(parent), old);
    }
    gtk_container_add(GTK_CONTAINER(parent), child);
    *out = MAKE_BOOL(true);
}

void gui_add(SnaskValue* out, SnaskValue* box_h, SnaskValue* child_h) {
    if ((int)box_h->tag != SNASK_STR || (int)child_h->tag != SNASK_STR) { *out = MAKE_NIL(); return; }
    GtkWidget* box = (GtkWidget*)gui_handle_to_ptr((const char*)box_h->ptr);
    GtkWidget* child = (GtkWidget*)gui_handle_to_ptr((const char*)child_h->ptr);
    if (!box || !child) { *out = MAKE_NIL(); return; }
    if (GTK_IS_BOX(box)) gtk_box_pack_start(GTK_BOX(box), child, FALSE, FALSE, 0);
    else if (GTK_IS_CONTAINER(box)) gtk_container_add(GTK_CONTAINER(box), child);
    else { *out = MAKE_NIL(); return; }
    *out = MAKE_BOOL(true);
}

void gui_add_expand(SnaskValue* out, SnaskValue* box_h, SnaskValue* child_h) {
    if ((int)box_h->tag != SNASK_STR || (int)child_h->tag != SNASK_STR) { *out = MAKE_NIL(); return; }
    GtkWidget* box = (GtkWidget*)gui_handle_to_ptr((const char*)box_h->ptr);
    GtkWidget* child = (GtkWidget*)gui_handle_to_ptr((const char*)child_h->ptr);
    if (!box || !child) { *out = MAKE_NIL(); return; }
    if (GTK_IS_BOX(box)) gtk_box_pack_start(GTK_BOX(box), child, TRUE, TRUE, 0);
    else if (GTK_IS_CONTAINER(box)) gtk_container_add(GTK_CONTAINER(box), child);
    else { *out = MAKE_NIL(); return; }
    *out = MAKE_BOOL(true);
}

void gui_label(SnaskValue* out, SnaskValue* text) {
    if ((int)text->tag != SNASK_STR) { *out = MAKE_NIL(); return; }
    GtkWidget* w = gtk_label_new((const char*)text->ptr);
    *out = MAKE_STR(gui_ptr_to_handle(w));
}

void gui_entry(SnaskValue* out) {
    GtkWidget* e = gtk_entry_new();
    *out = MAKE_STR(gui_ptr_to_handle(e));
}

void gui_textview(SnaskValue* out) {
    GtkWidget* tv = gtk_text_view_new();
    *out = MAKE_STR(gui_ptr_to_handle(tv));
}

void gui_set_placeholder(SnaskValue* out, SnaskValue* entry_h, SnaskValue* text) {
    if ((int)entry_h->tag != SNASK_STR || (int)text->tag != SNASK_STR) { *out = MAKE_NIL(); return; }
    GtkWidget* w = (GtkWidget*)gui_handle_to_ptr((const char*)entry_h->ptr);
    if (!w || !GTK_IS_ENTRY(w)) { *out = MAKE_NIL(); return; }
    gtk_entry_set_placeholder_text(GTK_ENTRY(w), (const char*)text->ptr);
    *out = MAKE_BOOL(true);
}

void gui_set_editable(SnaskValue* out, SnaskValue* entry_h, SnaskValue* editable) {
    if ((int)entry_h->tag != SNASK_STR || (int)editable->tag != SNASK_BOOL) { *out = MAKE_NIL(); return; }
    GtkWidget* w = (GtkWidget*)gui_handle_to_ptr((const char*)entry_h->ptr);
    if (!w || !GTK_IS_ENTRY(w)) { *out = MAKE_NIL(); return; }
    gtk_editable_set_editable(GTK_EDITABLE(w), editable->num != 0.0);
    *out = MAKE_BOOL(true);
}

void gui_button(SnaskValue* out, SnaskValue* text) {
    if ((int)text->tag != SNASK_STR) { *out = MAKE_NIL(); return; }
    GtkWidget* b = gtk_button_new_with_label((const char*)text->ptr);
    *out = MAKE_STR(gui_ptr_to_handle(b));
}

void gui_set_enabled(SnaskValue* out, SnaskValue* widget_h, SnaskValue* enabled) {
    if ((int)widget_h->tag != SNASK_STR || (int)enabled->tag != SNASK_BOOL) { *out = MAKE_NIL(); return; }
    GtkWidget* w = (GtkWidget*)gui_handle_to_ptr((const char*)widget_h->ptr);
    if (!w) { *out = MAKE_NIL(); return; }
    gtk_widget_set_sensitive(w, enabled->num != 0.0);
    *out = MAKE_BOOL(true);
}

void gui_set_visible(SnaskValue* out, SnaskValue* widget_h, SnaskValue* visible) {
    if ((int)widget_h->tag != SNASK_STR || (int)visible->tag != SNASK_BOOL) { *out = MAKE_NIL(); return; }
    GtkWidget* w = (GtkWidget*)gui_handle_to_ptr((const char*)widget_h->ptr);
    if (!w) { *out = MAKE_NIL(); return; }
    gtk_widget_set_visible(w, visible->num != 0.0);
    *out = MAKE_BOOL(true);
}

void gui_show_all(SnaskValue* out, SnaskValue* widget_h) {
    if ((int)widget_h->tag != SNASK_STR) { *out = MAKE_NIL(); return; }
    GtkWidget* w = (GtkWidget*)gui_handle_to_ptr((const char*)widget_h->ptr);
    if (!w) { *out = MAKE_NIL(); return; }
    gtk_widget_show_all(w);
    *out = MAKE_NIL();
}

void gui_set_text(SnaskValue* out, SnaskValue* widget_h, SnaskValue* text) {
    if ((int)widget_h->tag != SNASK_STR || (int)text->tag != SNASK_STR) { *out = MAKE_NIL(); return; }
    GtkWidget* w = (GtkWidget*)gui_handle_to_ptr((const char*)widget_h->ptr);
    if (!w) { *out = MAKE_NIL(); return; }
    if (GTK_IS_LABEL(w)) gtk_label_set_text(GTK_LABEL(w), (const char*)text->ptr);
    else if (GTK_IS_BUTTON(w)) gtk_button_set_label(GTK_BUTTON(w), (const char*)text->ptr);
    else if (GTK_IS_ENTRY(w)) gtk_entry_set_text(GTK_ENTRY(w), (const char*)text->ptr);
    else if (GTK_IS_TEXT_VIEW(w)) {
        GtkTextBuffer* buf = gtk_text_view_get_buffer(GTK_TEXT_VIEW(w));
        gtk_text_buffer_set_text(buf, (const char*)text->ptr, -1);
    }
    *out = MAKE_BOOL(true);
}

void gui_get_text(SnaskValue* out, SnaskValue* widget_h) {
    if ((int)widget_h->tag != SNASK_STR) { *out = MAKE_NIL(); return; }
    GtkWidget* w = (GtkWidget*)gui_handle_to_ptr((const char*)widget_h->ptr);
    if (!w) { *out = MAKE_NIL(); return; }
    if (GTK_IS_ENTRY(w)) {
        const char* t = gtk_entry_get_text(GTK_ENTRY(w));
        *out = MAKE_STR(snask_gc_strdup(t ? t : ""));
        return;
    }
    if (GTK_IS_TEXT_VIEW(w)) {
        GtkTextBuffer* buf = gtk_text_view_get_buffer(GTK_TEXT_VIEW(w));
        GtkTextIter start, end;
        gtk_text_buffer_get_bounds(buf, &start, &end);
        char* t = gtk_text_buffer_get_text(buf, &start, &end, TRUE);
        *out = MAKE_STR(snask_gc_strdup(t ? t : ""));
        if (t) g_free(t);
        return;
    }
    *out = MAKE_STR(snask_gc_strdup(""));
}

void gui_on_click(SnaskValue* out, SnaskValue* widget_h, SnaskValue* handler_name) {
    if ((int)widget_h->tag != SNASK_STR || (int)handler_name->tag != SNASK_STR) { *out = MAKE_NIL(); return; }
    GtkWidget* w = (GtkWidget*)gui_handle_to_ptr((const char*)widget_h->ptr);
    if (!w || !GTK_IS_BUTTON(w)) { *out = MAKE_NIL(); return; }
    GuiCallbackCtx* ctx = (GuiCallbackCtx*)calloc(1, sizeof(GuiCallbackCtx));
    ctx->handler_name = strdup((const char*)handler_name->ptr);
    ctx->widget_handle = strdup((const char*)widget_h->ptr);
    ctx->ctx = NULL;
    g_signal_connect_data(w, "clicked", G_CALLBACK(gui_on_button_clicked), ctx, (GClosureNotify)gui_free_ctx, 0);
    *out = MAKE_BOOL(true);
}

void gui_on_click_ctx(SnaskValue* out, SnaskValue* widget_h, SnaskValue* handler_name, SnaskValue* ctx_str) {
    if ((int)widget_h->tag != SNASK_STR || (int)handler_name->tag != SNASK_STR || (int)ctx_str->tag != SNASK_STR) { *out = MAKE_NIL(); return; }
    GtkWidget* w = (GtkWidget*)gui_handle_to_ptr((const char*)widget_h->ptr);
    if (!w || !GTK_IS_BUTTON(w)) { *out = MAKE_NIL(); return; }
    GuiCallbackCtx* ctx = (GuiCallbackCtx*)calloc(1, sizeof(GuiCallbackCtx));
    ctx->handler_name = strdup((const char*)handler_name->ptr);
    ctx->widget_handle = strdup((const char*)widget_h->ptr);
    ctx->ctx = strdup((const char*)ctx_str->ptr);
    g_signal_connect_data(w, "clicked", G_CALLBACK(gui_on_button_clicked), ctx, (GClosureNotify)gui_free_ctx, 0);
    *out = MAKE_BOOL(true);
}

void gui_separator_h(SnaskValue* out) {
    GtkWidget* s = gtk_separator_new(GTK_ORIENTATION_HORIZONTAL);
    *out = MAKE_STR(gui_ptr_to_handle(s));
}

void gui_separator_v(SnaskValue* out) {
    GtkWidget* s = gtk_separator_new(GTK_ORIENTATION_VERTICAL);
    *out = MAKE_STR(gui_ptr_to_handle(s));
}

static void gui_msg_dialog(GtkMessageType t, const char* title, const char* msg) {
    GtkWidget* dialog = gtk_message_dialog_new(NULL, GTK_DIALOG_MODAL, t, GTK_BUTTONS_OK, "%s", msg ? msg : "");
    if (title) gtk_window_set_title(GTK_WINDOW(dialog), title);
    gtk_dialog_run(GTK_DIALOG(dialog));
    gtk_widget_destroy(dialog);
}

void gui_msg_info(SnaskValue* out, SnaskValue* title, SnaskValue* msg) {
    if ((int)title->tag != SNASK_STR || (int)msg->tag != SNASK_STR) { *out = MAKE_NIL(); return; }
    gui_msg_dialog(GTK_MESSAGE_INFO, (const char*)title->ptr, (const char*)msg->ptr);
    *out = MAKE_NIL();
}

void gui_msg_error(SnaskValue* out, SnaskValue* title, SnaskValue* msg) {
    if ((int)title->tag != SNASK_STR || (int)msg->tag != SNASK_STR) { *out = MAKE_NIL(); return; }
    gui_msg_dialog(GTK_MESSAGE_ERROR, (const char*)title->ptr, (const char*)msg->ptr);
    *out = MAKE_NIL();
}

// --- Drawing API (Cairo/Skia) ---
typedef struct {
    int w; int h;
    double r, g, b, a;
    cairo_surface_t* surface;
    cairo_t* cr;
} SnaskSkiaSurface;

static SnaskSkiaSurface** skia_surfaces = NULL;
static size_t skia_surfaces_len = 0;
static size_t skia_surfaces_cap = 0;

static void skia_track_surface(SnaskSkiaSurface* s) {
    if (!s) return;
    if (skia_surfaces_len == skia_surfaces_cap) {
        size_t nc = skia_surfaces_cap ? skia_surfaces_cap * 2 : 64;
        SnaskSkiaSurface** n = (SnaskSkiaSurface**)realloc(skia_surfaces, nc * sizeof(SnaskSkiaSurface*));
        if (!n) return;
        skia_surfaces = n;
        skia_surfaces_cap = nc;
    }
    skia_surfaces[skia_surfaces_len++] = s;
}

static SnaskSkiaSurface* skia_get_surface(const char* handle) {
    if (!handle) return NULL;
    const char* pfx = "skia_surface:cairo:";
    if (strncmp(handle, pfx, strlen(pfx)) != 0) return NULL;
    size_t idx = (size_t)strtol(handle + strlen(pfx), NULL, 10);
    if (idx >= skia_surfaces_len) return NULL;
    return skia_surfaces[idx];
}

#ifdef SNASK_SKIA
#include "skia_bridge.h"
static int snask_skia_default_backend = 0;
void skia_use_real(SnaskValue* out, SnaskValue* enabled) {
    snask_skia_default_backend = (enabled->num != 0.0) ? 1 : 0;
    *out = MAKE_BOOL(true);
}
void skia_version(SnaskValue* out) { *out = MAKE_STR(snask_gc_strdup(snask_skia_impl_version())); }
#else
void skia_use_real(SnaskValue* out, SnaskValue* enabled) { *out = MAKE_BOOL(false); }
void skia_version(SnaskValue* out) { *out = MAKE_STR(snask_gc_strdup("cairo-backend")); }
#endif

void skia_surface(SnaskValue* out, SnaskValue* wv, SnaskValue* hv) {
    if ((int)wv->tag != SNASK_NUM || (int)hv->tag != SNASK_NUM) { *out = MAKE_NIL(); return; }
    int w = (int)wv->num, h = (int)hv->num;
#ifdef SNASK_SKIA
    if (snask_skia_default_backend == 1) {
        int id = snask_skia_impl_surface_create(w, h);
        char buf[64]; snprintf(buf, 64, "skia_surface:skia:%d", id);
        *out = MAKE_STR(snask_gc_strdup(buf)); return;
    }
#endif
    SnaskSkiaSurface* s = (SnaskSkiaSurface*)calloc(1, sizeof(SnaskSkiaSurface));
    s->w = w; s->h = h; s->r = 1.0; s->g = 1.0; s->b = 1.0; s->a = 1.0;
    s->surface = cairo_image_surface_create(CAIRO_FORMAT_ARGB32, w, h);
    s->cr = cairo_create(s->surface);
    skia_track_surface(s);
    char buf[64]; snprintf(buf, 64, "skia_surface:cairo:%zu", skia_surfaces_len - 1);
    *out = MAKE_STR(snask_gc_strdup(buf));
}

static int skia_parse_handle(const char* handle, bool* is_skia) {
    if (strncmp(handle, "skia_surface:skia:", 18) == 0) { if (is_skia) *is_skia = true; return atoi(handle + 18); }
    if (strncmp(handle, "skia_surface:cairo:", 19) == 0) { if (is_skia) *is_skia = false; return atoi(handle + 19); }
    return -1;
}

void skia_surface_width(SnaskValue* out, SnaskValue* surface_h) {
    bool is_skia = false; int id = skia_parse_handle((const char*)surface_h->ptr, &is_skia);
#ifdef SNASK_SKIA
    if (is_skia) { *out = MAKE_NUM((double)snask_skia_impl_surface_width(id)); return; }
#endif
    SnaskSkiaSurface* s = skia_get_surface((const char*)surface_h->ptr);
    *out = s ? MAKE_NUM((double)s->w) : MAKE_NIL();
}

void skia_surface_height(SnaskValue* out, SnaskValue* surface_h) {
    bool is_skia = false; int id = skia_parse_handle((const char*)surface_h->ptr, &is_skia);
#ifdef SNASK_SKIA
    if (is_skia) { *out = MAKE_NUM((double)snask_skia_impl_surface_height(id)); return; }
#endif
    SnaskSkiaSurface* s = skia_get_surface((const char*)surface_h->ptr);
    *out = s ? MAKE_NUM((double)s->h) : MAKE_NIL();
}

void skia_surface_clear(SnaskValue* out, SnaskValue* surface_h, SnaskValue* rv, SnaskValue* gv, SnaskValue* bv, SnaskValue* av) {
    bool is_skia = false; int id = skia_parse_handle((const char*)surface_h->ptr, &is_skia);
#ifdef SNASK_SKIA
    if (is_skia) { *out = MAKE_BOOL(snask_skia_impl_surface_clear(id, rv->num, gv->num, bv->num, av->num)); return; }
#endif
    SnaskSkiaSurface* s = skia_get_surface((const char*)surface_h->ptr);
    if (s && s->cr) {
        cairo_save(s->cr); cairo_set_source_rgba(s->cr, rv->num, gv->num, bv->num, av->num);
        cairo_set_operator(s->cr, CAIRO_OPERATOR_SOURCE); cairo_paint(s->cr); cairo_restore(s->cr);
        *out = MAKE_BOOL(true);
    } else *out = MAKE_NIL();
}

void skia_surface_set_color(SnaskValue* out, SnaskValue* surface_h, SnaskValue* rv, SnaskValue* gv, SnaskValue* bv, SnaskValue* av) {
    bool is_skia = false; int id = skia_parse_handle((const char*)surface_h->ptr, &is_skia);
#ifdef SNASK_SKIA
    if (is_skia) { *out = MAKE_BOOL(snask_skia_impl_surface_set_color(id, rv->num, gv->num, bv->num, av->num)); return; }
#endif
    SnaskSkiaSurface* s = skia_get_surface((const char*)surface_h->ptr);
    if (s) { s->r = rv->num; s->g = gv->num; s->b = bv->num; s->a = av->num; *out = MAKE_BOOL(true); } else *out = MAKE_NIL();
}

void skia_draw_rect(SnaskValue* out, SnaskValue* surface_h, SnaskValue* xv, SnaskValue* yv, SnaskValue* wv, SnaskValue* hv, SnaskValue* fillv) {
    bool is_skia = false; int id = skia_parse_handle((const char*)surface_h->ptr, &is_skia);
#ifdef SNASK_SKIA
    if (is_skia) { *out = MAKE_BOOL(snask_skia_impl_draw_rect(id, xv->num, yv->num, wv->num, hv->num, fillv->num != 0.0)); return; }
#endif
    SnaskSkiaSurface* s = skia_get_surface((const char*)surface_h->ptr);
    if (s && s->cr) {
        cairo_set_source_rgba(s->cr, s->r, s->g, s->b, s->a);
        cairo_rectangle(s->cr, xv->num, yv->num, wv->num, hv->num);
        if (fillv->num != 0.0) cairo_fill(s->cr); else cairo_stroke(s->cr);
        *out = MAKE_BOOL(true);
    } else *out = MAKE_NIL();
}

void skia_draw_circle(SnaskValue* out, SnaskValue* surface_h, SnaskValue* cxv, SnaskValue* cyv, SnaskValue* rv, SnaskValue* fillv) {
    bool is_skia = false; int id = skia_parse_handle((const char*)surface_h->ptr, &is_skia);
#ifdef SNASK_SKIA
    if (is_skia) { *out = MAKE_BOOL(snask_skia_impl_draw_circle(id, cxv->num, cyv->num, rv->num, fillv->num != 0.0)); return; }
#endif
    SnaskSkiaSurface* s = skia_get_surface((const char*)surface_h->ptr);
    if (s && s->cr) {
        cairo_set_source_rgba(s->cr, s->r, s->g, s->b, s->a);
        cairo_arc(s->cr, cxv->num, cyv->num, rv->num, 0.0, 2.0 * M_PI);
        if (fillv->num != 0.0) cairo_fill(s->cr); else cairo_stroke(s->cr);
        *out = MAKE_BOOL(true);
    } else *out = MAKE_NIL();
}

void skia_draw_line(SnaskValue* out, SnaskValue* surface_h, SnaskValue* x1v, SnaskValue* y1v, SnaskValue* x2v, SnaskValue* y2v, SnaskValue* stroke_wv) {
    bool is_skia = false; int id = skia_parse_handle((const char*)surface_h->ptr, &is_skia);
#ifdef SNASK_SKIA
    if (is_skia) { *out = MAKE_BOOL(snask_skia_impl_draw_line(id, x1v->num, y1v->num, x2v->num, y2v->num, stroke_wv->num)); return; }
#endif
    SnaskSkiaSurface* s = skia_get_surface((const char*)surface_h->ptr);
    if (s && s->cr) {
        cairo_set_source_rgba(s->cr, s->r, s->g, s->b, s->a); cairo_set_line_width(s->cr, stroke_wv->num);
        cairo_move_to(s->cr, x1v->num, y1v->num); cairo_line_to(s->cr, x2v->num, y2v->num); cairo_stroke(s->cr);
        *out = MAKE_BOOL(true);
    } else *out = MAKE_NIL();
}

void skia_draw_text(SnaskValue* out, SnaskValue* surface_h, SnaskValue* xv, SnaskValue* yv, SnaskValue* textv, SnaskValue* sizev) {
    bool is_skia = false; int id = skia_parse_handle((const char*)surface_h->ptr, &is_skia);
#ifdef SNASK_SKIA
    if (is_skia) { *out = MAKE_BOOL(snask_skia_impl_draw_text(id, xv->num, yv->num, (const char*)textv->ptr, sizev->num)); return; }
#endif
    SnaskSkiaSurface* s = skia_get_surface((const char*)surface_h->ptr);
    if (s && s->cr) {
        cairo_set_source_rgba(s->cr, s->r, s->g, s->b, s->a); cairo_set_font_size(s->cr, sizev->num);
        cairo_move_to(s->cr, xv->num, yv->num); cairo_show_text(s->cr, (const char*)textv->ptr);
        *out = MAKE_BOOL(true);
    } else *out = MAKE_NIL();
}

void skia_save_png(SnaskValue* out, SnaskValue* surface_h, SnaskValue* pathv) {
    bool is_skia = false; int id = skia_parse_handle((const char*)surface_h->ptr, &is_skia);
#ifdef SNASK_SKIA
    if (is_skia) { *out = MAKE_BOOL(snask_skia_impl_save_png(id, (const char*)pathv->ptr)); return; }
#endif
    SnaskSkiaSurface* s = skia_get_surface((const char*)surface_h->ptr);
    if (s && s->surface) *out = MAKE_BOOL(cairo_surface_write_to_png(s->surface, (const char*)pathv->ptr) == CAIRO_STATUS_SUCCESS);
    else *out = MAKE_NIL();
}

void snaskgui_init(SnaskValue* out) {
    int argc = 0;
    char** argv = NULL;
    gboolean ok = gtk_init_check(&argc, &argv);
    *out = MAKE_BOOL(ok);
}

void snaskgui_window(SnaskValue* out, SnaskValue* title, SnaskValue* w, SnaskValue* h, SnaskValue* scale) {
    if ((int)title->tag != SNASK_STR || (int)w->tag != SNASK_NUM || (int)h->tag != SNASK_NUM || (int)scale->tag != SNASK_NUM) {
        *out = MAKE_NIL();
        return;
    }
    SnaskGuiWindow* win = (SnaskGuiWindow*)calloc(1, sizeof(SnaskGuiWindow));
    win->width = (int)w->num;
    win->height = (int)h->num;
    win->scale = (int)scale->num;
    if (win->scale <= 0) win->scale = 1;
    win->pixels = (unsigned char*)calloc((size_t)win->width * (size_t)win->height * 4, 1);

    win->window = gtk_window_new(GTK_WINDOW_TOPLEVEL);
    gtk_window_set_title(GTK_WINDOW(win->window), (const char*)title->ptr);
    gtk_window_set_default_size(GTK_WINDOW(win->window), win->width * win->scale, win->height * win->scale);
    gtk_window_set_resizable(GTK_WINDOW(win->window), FALSE);

    win->area = gtk_drawing_area_new();
    gtk_widget_set_size_request(win->area, win->width * win->scale, win->height * win->scale);
    gtk_container_add(GTK_CONTAINER(win->window), win->area);

    gtk_widget_add_events(win->window, GDK_KEY_PRESS_MASK | GDK_KEY_RELEASE_MASK);
    gtk_widget_add_events(win->area, GDK_POINTER_MOTION_MASK | GDK_BUTTON_PRESS_MASK | GDK_BUTTON_RELEASE_MASK);
    
    g_signal_connect(win->area, "draw", G_CALLBACK(snaskgui_draw_cb), win);
    g_signal_connect(win->window, "key-press-event", G_CALLBACK(snaskgui_key_press_cb), win);
    g_signal_connect(win->window, "key-release-event", G_CALLBACK(snaskgui_key_release_cb), win);
    g_signal_connect(win->area, "motion-notify-event", G_CALLBACK(snaskgui_motion_cb), win);
    g_signal_connect(win->area, "button-press-event", G_CALLBACK(snaskgui_button_cb), win);
    g_signal_connect(win->area, "button-release-event", G_CALLBACK(snaskgui_button_cb), win);
    g_signal_connect(win->window, "destroy", G_CALLBACK(snaskgui_destroy_cb), win);
    gtk_widget_show_all(win->window);

    *out = MAKE_STR(gui_ptr_to_handle(win));
}

void snaskgui_mouse_x(SnaskValue* out, SnaskValue* win_h) {
    SnaskGuiWindow* win = snaskgui_from_handle(win_h);
    *out = MAKE_NUM(win ? win->mouse_x : 0.0);
}

void snaskgui_mouse_y(SnaskValue* out, SnaskValue* win_h) {
    SnaskGuiWindow* win = snaskgui_from_handle(win_h);
    *out = MAKE_NUM(win ? win->mouse_y : 0.0);
}

void snaskgui_mouse_down(SnaskValue* out, SnaskValue* win_h, SnaskValue* btn) {
    SnaskGuiWindow* win = snaskgui_from_handle(win_h);
    if (!win || (int)btn->tag != SNASK_NUM) { *out = MAKE_BOOL(false); return; }
    int b = (int)btn->num;
    *out = MAKE_BOOL(b >= 0 && b < 5 && win->mouse_btns[b] != 0);
}

void snaskgui_present_rgba(SnaskValue* out, SnaskValue* win_h, SnaskValue* pixels, SnaskValue* w, SnaskValue* h) {
    SnaskGuiWindow* win = snaskgui_from_handle(win_h);
    if (!win || !pixels || (int)w->tag != SNASK_NUM || (int)h->tag != SNASK_NUM) {
        *out = MAKE_BOOL(false);
        return;
    }
    int pw = (int)w->num;
    int ph = (int)h->num;
    if (pw != win->width || ph != win->height || !pixels->ptr) {
        *out = MAKE_BOOL(false);
        return;
    }
    memcpy(win->pixels, pixels->ptr, (size_t)win->width * (size_t)win->height * 4);
    gtk_widget_queue_draw(win->area);
    *out = MAKE_BOOL(true);
}

void snaskgui_poll(SnaskValue* out, SnaskValue* win_h) {
    SnaskGuiWindow* win = snaskgui_from_handle(win_h);
    while (gtk_events_pending()) gtk_main_iteration_do(FALSE);
    *out = MAKE_BOOL(win && !win->should_close);
}

void snaskgui_key_down(SnaskValue* out, SnaskValue* win_h, SnaskValue* key) {
    SnaskGuiWindow* win = snaskgui_from_handle(win_h);
    if (!win || (int)key->tag != SNASK_NUM) {
        *out = MAKE_BOOL(false);
        return;
    }
    int idx = (int)key->num;
    *out = MAKE_BOOL(idx >= 0 && idx < 512 && win->keys[idx] != 0);
}

void snaskgui_should_close(SnaskValue* out, SnaskValue* win_h) {
    SnaskGuiWindow* win = snaskgui_from_handle(win_h);
    *out = MAKE_BOOL(!win || win->should_close != 0);
}

void snaskgui_delay(SnaskValue* out, SnaskValue* ms) {
    if ((int)ms->tag == SNASK_NUM && ms->num > 0) {
        g_usleep((gulong)(ms->num * 1000.0));
    }
    *out = MAKE_NIL();
}

void snaskgui_close(SnaskValue* out, SnaskValue* win_h) {
    SnaskGuiWindow* win = snaskgui_from_handle(win_h);
    if (win) {
        win->should_close = 1;
        if (win->window) gtk_widget_destroy(win->window);
    }
    *out = MAKE_NIL();
}

#else
// Stubs when GTK is not enabled
void gui_init(SnaskValue* out) { *out = MAKE_BOOL(false); }
void gui_run(SnaskValue* out) { *out = MAKE_NIL(); }
void gui_quit(SnaskValue* out) { *out = MAKE_NIL(); }
void snaskgui_init(SnaskValue* out) { *out = MAKE_BOOL(false); }
void snaskgui_window(SnaskValue* out, SnaskValue* title, SnaskValue* w, SnaskValue* h, SnaskValue* scale) { (void)title; (void)w; (void)h; (void)scale; *out = MAKE_NIL(); }
void snaskgui_present_rgba(SnaskValue* out, SnaskValue* win_h, SnaskValue* pixels, SnaskValue* w, SnaskValue* h) { (void)win_h; (void)pixels; (void)w; (void)h; *out = MAKE_BOOL(false); }
void snaskgui_poll(SnaskValue* out, SnaskValue* win_h) { (void)win_h; *out = MAKE_BOOL(false); }
void snaskgui_key_down(SnaskValue* out, SnaskValue* win_h, SnaskValue* key) { (void)win_h; (void)key; *out = MAKE_BOOL(false); }
void snaskgui_should_close(SnaskValue* out, SnaskValue* win_h) { (void)win_h; *out = MAKE_BOOL(true); }
void snaskgui_delay(SnaskValue* out, SnaskValue* ms) { (void)ms; *out = MAKE_NIL(); }
void snaskgui_close(SnaskValue* out, SnaskValue* win_h) { (void)win_h; *out = MAKE_NIL(); }
// ... (rest of stubs omitted for brevity, but would be in the full file)
#endif
