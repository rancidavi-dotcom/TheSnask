#ifndef RT_GUI_H
#define RT_GUI_H

#include "rt_base.h"

// GUI Initialization and Lifecycle
void gui_init(SnaskValue* out);
void gui_run(SnaskValue* out);
void gui_quit(SnaskValue* out);

// Widgets and Windows
void gui_window(SnaskValue* out, SnaskValue* title, SnaskValue* w, SnaskValue* h);
void gui_set_title(SnaskValue* out, SnaskValue* win_h, SnaskValue* title);
void gui_set_resizable(SnaskValue* out, SnaskValue* win_h, SnaskValue* resizable);
void gui_autosize(SnaskValue* out, SnaskValue* win_h);
void gui_vbox(SnaskValue* out);
void gui_hbox(SnaskValue* out);
void gui_eventbox(SnaskValue* out);
void gui_scrolled(SnaskValue* out);
void gui_flowbox(SnaskValue* out);
void gui_flow_add(SnaskValue* out, SnaskValue* flow_h, SnaskValue* child_h);
void gui_frame(SnaskValue* out);
void gui_set_margin(SnaskValue* out, SnaskValue* widget_h, SnaskValue* margin_v);
void gui_icon(SnaskValue* out, SnaskValue* name, SnaskValue* size_v);
void gui_css(SnaskValue* out, SnaskValue* css);
void gui_add_class(SnaskValue* out, SnaskValue* widget_h, SnaskValue* cls);
void gui_listbox(SnaskValue* out);
void gui_list_add_text(SnaskValue* out, SnaskValue* list_h, SnaskValue* text);
void gui_on_select_ctx(SnaskValue* out, SnaskValue* list_h, SnaskValue* handler_name, SnaskValue* ctx_str);
void gui_set_child(SnaskValue* out, SnaskValue* parent_h, SnaskValue* child_h);
void gui_add(SnaskValue* out, SnaskValue* box_h, SnaskValue* child_h);
void gui_add_expand(SnaskValue* out, SnaskValue* box_h, SnaskValue* child_h);
void gui_label(SnaskValue* out, SnaskValue* text);
void gui_entry(SnaskValue* out);
void gui_textview(SnaskValue* out);
void gui_set_placeholder(SnaskValue* out, SnaskValue* entry_h, SnaskValue* text);
void gui_set_editable(SnaskValue* out, SnaskValue* entry_h, SnaskValue* editable);
void gui_button(SnaskValue* out, SnaskValue* text);
void gui_set_enabled(SnaskValue* out, SnaskValue* widget_h, SnaskValue* enabled);
void gui_set_visible(SnaskValue* out, SnaskValue* widget_h, SnaskValue* visible);
void gui_show_all(SnaskValue* out, SnaskValue* widget_h);
void gui_set_text(SnaskValue* out, SnaskValue* widget_h, SnaskValue* text);
void gui_get_text(SnaskValue* out, SnaskValue* widget_h);
void gui_on_click(SnaskValue* out, SnaskValue* widget_h, SnaskValue* handler_name);
void gui_on_click_ctx(SnaskValue* out, SnaskValue* widget_h, SnaskValue* handler_name, SnaskValue* ctx_str);
void gui_on_tap_ctx(SnaskValue* out, SnaskValue* widget_h, SnaskValue* handler_name, SnaskValue* ctx_str);
void gui_separator_h(SnaskValue* out);
void gui_separator_v(SnaskValue* out);
void gui_msg_info(SnaskValue* out, SnaskValue* title, SnaskValue* msg);
void gui_msg_error(SnaskValue* out, SnaskValue* title, SnaskValue* msg);

// Skia / Drawing API
void skia_version(SnaskValue* out);
void skia_use_real(SnaskValue* out, SnaskValue* enabled);
void skia_surface(SnaskValue* out, SnaskValue* wv, SnaskValue* hv);
void skia_surface_width(SnaskValue* out, SnaskValue* surface_h);
void skia_surface_height(SnaskValue* out, SnaskValue* surface_h);
void skia_surface_clear(SnaskValue* out, SnaskValue* surface_h, SnaskValue* rv, SnaskValue* gv, SnaskValue* bv, SnaskValue* av);
void skia_surface_set_color(SnaskValue* out, SnaskValue* surface_h, SnaskValue* rv, SnaskValue* gv, SnaskValue* bv, SnaskValue* av);
void skia_draw_rect(SnaskValue* out, SnaskValue* surface_h, SnaskValue* xv, SnaskValue* yv, SnaskValue* wv, SnaskValue* hv, SnaskValue* fillv);
void skia_draw_circle(SnaskValue* out, SnaskValue* surface_h, SnaskValue* cxv, SnaskValue* cyv, SnaskValue* rv, SnaskValue* fillv);
void skia_draw_line(SnaskValue* out, SnaskValue* surface_h, SnaskValue* x1v, SnaskValue* y1v, SnaskValue* x2v, SnaskValue* y2v, SnaskValue* stroke_wv);
void skia_draw_text(SnaskValue* out, SnaskValue* surface_h, SnaskValue* xv, SnaskValue* yv, SnaskValue* textv, SnaskValue* sizev);
void skia_save_png(SnaskValue* out, SnaskValue* surface_h, SnaskValue* pathv);

#endif // RT_GUI_H
