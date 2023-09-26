fn color_box_ui(ui: &mut egui::Ui, current_value: &mut [u8; 3], selection_value: [u8; 3]) -> egui::Response {
    let mut size = egui::vec2(1.0, 1.0);
    let selected = *current_value == selection_value;
    if selected {
        size = egui::vec2(1.2, 1.2);
    }
    let desired_size = ui.spacing().interact_size.y * size;
    let (mut rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
    if response.clicked() {
        *current_value = selection_value;
        response.mark_changed();
    }

    if ui.is_rect_visible(rect) {
        let mut stroke_color = egui::Color32::GRAY;
        let mut stroke_width = 2.0;

        let visuals = ui.style().interact_selectable(&response, selected);

        if !selected {
            rect = rect.expand(visuals.expansion);
        }

        if selected {
            stroke_color = egui::Color32::LIGHT_BLUE;
            stroke_width = 3.0;
        }
        ui.painter().rect(rect, 0.0, egui::Color32::from_rgb(selection_value[0], selection_value[1], selection_value[2]), egui::Stroke::new(stroke_width, stroke_color));
    }

    response
}

pub fn color_box(current_value: &mut [u8; 3], alternative: [u8; 3]) -> impl egui::Widget + '_ {
    move |ui: &mut egui::Ui| color_box_ui(ui, current_value, alternative)
}
