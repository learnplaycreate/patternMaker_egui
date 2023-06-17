use std::sync::{Arc, RwLock};
use epaint::{Vec2, Pos2, Shape, Stroke, Color32};
use pm_pattern_logic::{NotificationHandler, PatternPos, Pattern, PatternElement, CurrentDrawingTool, SelectedElements, ToolKind, get_selected_items_length, AxisKind, set_render_to_index};

use crate::Response;

use super::{ScreenTransform, PlotPoint};


pub struct PmEguiPlotHelpers {
    pub pattern : Arc<RwLock<Pattern>>, 
    pub drawing_tool : Arc<RwLock<CurrentDrawingTool>>, 
    pub selected_items : Arc<RwLock<SelectedElements>>, 
    pub js_helpers : Arc<RwLock<NotificationHandler>>,
    response_drag_delta_detection_limit : f32, // RESPONSE_DRAG_DELTA_DETECTION_LIMIT
    pattern_drawing_live_colour: Color32,
    pattern_measurement_colour: Color32,
    pattern_line_stroke_width: f32,
}

impl std::fmt::Debug for PmEguiPlotHelpers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{:.1}]", "need to practice".to_string())
    }
}

impl Default for PmEguiPlotHelpers {
    fn default() -> Self {
        Self {
            pattern :  Arc::new(RwLock::new(Pattern::default())),
            drawing_tool : Arc::new(RwLock::new(CurrentDrawingTool::default())),
            selected_items : Arc::new(RwLock::new(SelectedElements::default())),
            js_helpers : Arc::new(RwLock::new(NotificationHandler::default())),
            response_drag_delta_detection_limit: 0.0,
            pattern_drawing_live_colour: Color32::GREEN,
            pattern_measurement_colour: Color32::GRAY,
            pattern_line_stroke_width: 3.0,
        }
    }
}

impl PmEguiPlotHelpers{
    pub fn new(
        pattern : Arc<RwLock<Pattern>>, 
        drawing_tool : Arc<RwLock<CurrentDrawingTool>>, 
        selected_items : Arc<RwLock<SelectedElements>>, 
        js_helpers : Arc<RwLock<NotificationHandler>>,
        response_drag_delta_detection_limit : f32, // RESPONSE_DRAG_DELTA_DETECTION_LIMIT
        pattern_drawing_live_colour: Color32,
        pattern_measurement_colour: Color32,
        pattern_line_stroke_width: f32,) -> Self {
        Self {
            pattern,
            drawing_tool,
            selected_items,
            js_helpers,
            response_drag_delta_detection_limit, // RESPONSE_DRAG_DELTA_DETECTION_LIMIT
            pattern_drawing_live_colour,
            pattern_measurement_colour,
            pattern_line_stroke_width,
        }
    }
    
    pub (crate) fn _scale_shape_size_from_f64(transform: &ScreenTransform, current_size : f64) -> f64{
        let value_point_1 = PlotPoint::new(0.0, 0.0);
        let value_point_2 = PlotPoint::new(current_size, current_size);
        let pos_1 = transform.position_from_point(&value_point_1);
        let pos_2 = transform.position_from_point(&value_point_2);
        (pos_2[0] - pos_1[0]) as f64
    }
    
    //lpc - todo, would be good to see if we can call something in the screen transform to do this rather than making all it's props public. 
    pub (crate) fn translate_pos_drag(transform: &ScreenTransform, mut delta_pos: Vec2) -> Vec2{
        if transform.x_centered {
            delta_pos.x = 0.;
        }
        if transform.y_centered {
            delta_pos.y = 0.;
        }
        delta_pos.x *= transform.dvalue_dpos()[0] as f32;
        delta_pos.y *= transform.dvalue_dpos()[1] as f32;
        return delta_pos;
    }

    pub fn transform_pattern_pos_to_plot_point(pos : PatternPos) -> PlotPoint{
        PlotPoint {
            x: pos.x,
            y: pos.y,
        }
    }
    
    pub fn transform_plot_point_to_pattern_pos(pos : PlotPoint) -> PatternPos{
        PatternPos {
            x: pos.x,
            y: pos.y,
        }
    }

    pub (crate) fn plot_click_handleing(
        &self,
        response : Response, 
        last_screen_transform : ScreenTransform
    )
    {
        //debug!("lpc - in helper click handling");
        if response.drag_started(){ // && response.drag_delta().length() > RESPONSE_DRAG_DELTA_DETECTION_LIMIT{
            //test there is a click position
            if let Some(pointer_pos) = response.interact_pointer_pos() {
                let render_to_index = self.pattern.read().unwrap().render_to_index.clone();
                let position_value_from_position = last_screen_transform.value_from_position(pointer_pos);
                let drag_pattern_pos = PatternPos{x: position_value_from_position.x, y: position_value_from_position.y};
                let mut closest_element: Option<Box<dyn PatternElement>> = None;
                
                if let Ok(the_elements) = self.pattern.read().unwrap().get_all_pattern_elements(){
                    //see if there is a closest point
                    if let Some(element) = self.pattern.read().unwrap().get_closest_point_element(the_elements.clone(), drag_pattern_pos, last_screen_transform.bounds().min(), last_screen_transform.bounds().max()){
                        closest_element = Some(element.clone());
                        
                    }else{
                        //see if there is a closest other element that's not a point. 
                        if let Some(element) = self.pattern.read().unwrap().get_closest_non_point_element(the_elements, drag_pattern_pos, last_screen_transform.bounds().min(), last_screen_transform.bounds().max()){
                            closest_element = Some(element.clone());
                        }
                    }
                }

                if let Some(element) = closest_element{
                    if let Err(some_error) = self.drawing_tool.read().unwrap().get_current_tool().item_selected(drag_pattern_pos, Arc::clone(&self.pattern), Arc::clone(&self.selected_items), element, render_to_index){
                        (self.js_helpers.read().unwrap().alert_function)(&*some_error.to_string());
                    }
                }else{
                    if let Err(some_error) = self.drawing_tool.read().unwrap().get_current_tool().click_no_selection(drag_pattern_pos, Arc::clone(&self.pattern), Arc::clone(&self.selected_items), render_to_index){
                        //debug!("lpc - in helper click handling - handling error");
                        (self.js_helpers.read().unwrap().alert_function)(&*some_error.to_string());
                    }
                }
                set_render_to_index(&self.pattern, render_to_index);
            } 
        }
    }

    pub (crate) fn plot_drag_handling(
        &self,
        response : Response, 
        last_screen_transform : ScreenTransform, 
    ) -> bool{
        //todo lpc this currently has a limit of 1 and I'm not even sure it's doing anyting, should test to find the limt that stops code running when it's not required. 
        
        //todo lpc this currently has a limit of 1 and I'm not even sure it's doing anyting, should test to find the limt that stops code running when it's not required. 
        if response.drag_delta().length() > self.response_drag_delta_detection_limit{
            //debug!("lpc - in helper drag handling");
            let delta = response.drag_delta(); 
            //debug!("lpc - testing 2");
            let tranformed_delta = Self::translate_pos_drag(&last_screen_transform, delta);
            //debug!("lpc - testing 3");
            let drag_ids_option_response =  self.drawing_tool.read().unwrap().get_current_tool().get_draggable_items(Arc::clone(&self.pattern), Arc::clone(&self.selected_items));
            //debug!("lpc - testing 4");
            match drag_ids_option_response{
                Ok(drag_ids_option) => {
                    //debug!("lpc - testing 5");
                    if let Some(drag_ids) = drag_ids_option{
                        //debug!("lpc - testing 6");
                        if drag_ids.len() > 0{
                            //debug!("lpc - testing 7");
                            let render_to_index = self.pattern.read().unwrap().render_to_index.clone();
                            //debug!("lpc - testing 8");
                            self.pattern.write().unwrap().move_points_by_drag(drag_ids, Self::transform_plot_point_to_pattern_pos(PlotPoint::new(tranformed_delta.x, tranformed_delta.y)), render_to_index);
                            //debug!("lpc - testing 9");
                            set_render_to_index(&self.pattern, render_to_index);
                        }
                        return true;
                    }else{
                        return false;
                    }
                }
                Err(error) => {
                    (self.js_helpers.read().unwrap().alert_function)(&error.to_string());
                    return true;
                }
            }
        }else{
            return false;
        }
        
    }

    pub (crate) fn get_shapes_for_live_hover_drawing(
        &self,
        transform: ScreenTransform,
        pointer: Pos2,
    ) -> Vec<Shape>{
        let mut shapes : Vec<Shape> = vec![];
        match self.drawing_tool.read().unwrap().get_current_tool().get_drawing_tool_kind(){
            ToolKind::Curve(_) |
            ToolKind::Dart => {
                if get_selected_items_length(&self.selected_items) == 1 {
                    if let Ok(returned_live_shape_option) = self.drawing_tool.read().unwrap().get_current_tool().get_live_drawing_position(Self::transform_plot_point_to_pattern_pos(transform.value_from_position(pointer)), Arc::clone(&self.pattern), Arc::clone(&self.selected_items)){
                        if let Some(returned_points) = returned_live_shape_option{
                            if returned_points.len() == 2{
                                let line = Shape::line(vec![transform.position_from_point(&Self::transform_pattern_pos_to_plot_point(returned_points[0])) , transform.position_from_point(&Self::transform_pattern_pos_to_plot_point(returned_points[1]))], Stroke::new(self.pattern_line_stroke_width, self.pattern_drawing_live_colour));
                                shapes.push(line);
                            }
                        }
                    }
                }
            }
            //todo - lpc this is not working. 
            ToolKind::GuideLine(axis_kind) => { 
                if let Ok(returned_live_shape_option) = self.drawing_tool.read().unwrap().get_current_tool().get_live_drawing_position(Self::transform_plot_point_to_pattern_pos(transform.value_from_position(pointer)), Arc::clone(&self.pattern), Arc::clone(&self.selected_items)){
                    if let Some(returned_points) = returned_live_shape_option{
                        if returned_points.len() == 1{
                            let start_point : Pos2;
                            let end_point : Pos2;
                            
                            match axis_kind{
                                AxisKind::Horizontal => {
                                    start_point = transform.position_from_point(&PlotPoint::new(transform.bounds().min()[0], returned_points[0].y));
                                    end_point = transform.position_from_point(&PlotPoint::new(transform.bounds().max()[0], returned_points[0].y));
                                }
                                AxisKind::Vertical => {
                                    start_point = transform.position_from_point(&PlotPoint::new(returned_points[0].x, transform.bounds().min()[1]));
                                    end_point = transform.position_from_point(&PlotPoint::new(returned_points[0].x, transform.bounds().max()[1]));
                                }
                                AxisKind::Both => {
                                    panic!("this shouldn't happen ln214")
                                }
                            }
                    
                            let points = vec![
                                    start_point,
                                    end_point
                                ];
                            let line = Shape::line(points,Stroke::new(self.pattern_line_stroke_width/2.0,self.pattern_measurement_colour));
                            shapes.push(line);
                        }
                    }
                }
            }
            ToolKind::Point(_) => {
                //do nothing
            }
            ToolKind::Select(_) => {
                //do nothing
            }
        }
        return shapes;
    }
    
}





