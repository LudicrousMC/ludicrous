use std::hash::Hash;

use crate::server::terrain_gen::func_deserialize::{
    DensityFnOutline, DensityFnOutlineType, DensityOutlineArgType,
};
use crate::server::util::lerp_f64;

use super::{DensityArg, DensityFn, DensityFnArgs};
use ahash::AHasher;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct SplineFn {
    spline: SplineArgs,
}

impl DensityFn for SplineFn {
    #[inline]
    fn compute(&self, args: &mut DensityFnArgs) -> f64 {
        self.spline.compute(args)
    }

    #[inline]
    fn compute_slice(&self, args: &mut DensityFnArgs, data: &mut Vec<f64>) {
        self.compute_slice_keep_cache(args, data);
    }

    #[inline]
    fn get_min(&self, args: &mut DensityFnArgs) -> f64 {
        self.spline.get_min(args)
    }

    #[inline]
    fn get_max(&self, args: &mut DensityFnArgs) -> f64 {
        self.spline.get_max(args)
    }

    fn get_tree_hash(&self, state: &mut ahash::AHasher) {
        "spline".hash(state);
        self.spline.get_tree_hash(state);
    }

    fn precompute_noise_instance(&self, dimension: &str) {
        self.spline.precompute_noise_instance(dimension);
    }

    fn generate_state(&self, dimension: &str, outline: &mut DensityFnOutline) {
        //outline.new_stack_frame(DensityFnOutlineType::Spline);
        //let mut nested_counts = outline.push_placeholder_nested_data_counts();
        //nested_counts.initialize_nested_data_counts(outline);
        self.spline.generate_state(dimension, outline);
        //outline.apply_nested_data_counts(nested_counts);
    }
}

#[derive(Deserialize, Debug)]
struct SplineArgs {
    coordinate: SplineCoordinate,
    points: Vec<SplinePoint>,
}

impl SplineArgs {
    #[inline]
    fn compute(&self, args: &mut DensityFnArgs) -> f64 {
        let coord_val = self.coordinate.compute(args);
        let mut point_index = 0;
        let mut len = self.points.len() as i32;
        // Search self.points for location less than coordinate value
        while len > 0 {
            let half = len / 2;
            let mid = point_index + half;
            if coord_val < self.points[mid as usize].location {
                len = half;
            } else {
                point_index = mid + 1;
                len -= half + 1;
            }
        }
        point_index -= 1;
        if point_index < 0 {
            let first_point = &self.points[0];
            Self::linear_ext_if_non_zero(first_point, coord_val, first_point.value.compute(args))
        } else if point_index == self.points.len() as i32 - 1 {
            let last_point = &self.points[self.points.len() - 1];
            Self::linear_ext_if_non_zero(last_point, coord_val, last_point.value.compute(args))
        } else {
            let point = &self.points[point_index as usize];
            let next_point = &self.points[point_index as usize + 1];
            let point_distance = next_point.location - point.location;
            let pos = (coord_val - point.location) / point_distance;
            let point_val = point.value.compute(args);
            let next_point_val = next_point.value.compute(args);
            let point_val_distance = next_point_val - point_val;
            let val1 = point.derivative * point_distance - point_val_distance;
            let val2 = -next_point.derivative * point_distance + point_val_distance;
            lerp_f64(pos, point_val, next_point_val)
                + (pos * (1.0 - pos) * lerp_f64(pos, val1, val2))
        }
    }

    #[inline]
    fn get_min(&self, args: &mut DensityFnArgs) -> f64 {
        let mut min = f64::INFINITY;
        let coord_min = self.coordinate.get_min(args);
        let first_point = &self.points[0];
        if coord_min < first_point.location {
            let (linear_min, linear_max) = Self::linear_ext_min_max(first_point, coord_min, args);
            min = min.min(linear_min.min(linear_max));
        }

        let coord_max = self.coordinate.get_max(args);
        let last_point = &self.points[self.points.len() - 1];
        if coord_max > last_point.location {
            let (linear_min, linear_max) = Self::linear_ext_min_max(last_point, coord_max, args);
            min = min.min(linear_min.min(linear_max));
        }

        for point in self.points.iter() {
            min = min.min(point.value.get_min(args));
        }

        for i in 0..(self.points.len() - 1) {
            let point = &self.points[i];
            let next_point = &self.points[i + 1];
            let distance = next_point.location - point.location;
            let point_min = point.value.get_min(args);
            let next_point_min = next_point.value.get_min(args);
            // Calculate new min if at least one derivative is non-zero
            if point.derivative != 0.0 || next_point.derivative != 0.0 {
                let point_deriv_scaled = point.derivative * distance;
                let val1 = point_deriv_scaled - next_point.value.get_max(args) + point_min;
                let next_point_deriv_scaled = next_point.derivative * distance;
                let val2 = -next_point_deriv_scaled + next_point_min - point.value.get_max(args);
                let lowest = val1.min(val2);
                let points_min = point_min.min(next_point_min);
                min = min.min(points_min + (lowest * 0.25));
            }
        }
        min
    }

    #[inline]
    fn get_max(&self, args: &mut DensityFnArgs) -> f64 {
        let mut max = f64::NEG_INFINITY;
        let coord_min = self.coordinate.get_min(args);
        let first_point = &self.points[0];
        if coord_min < first_point.location {
            let (linear_min, linear_max) = Self::linear_ext_min_max(first_point, coord_min, args);
            max = max.max(linear_min.max(linear_max));
        }

        let coord_max = self.coordinate.get_max(args);
        let last_point = &self.points[self.points.len() - 1];
        if coord_max > last_point.location {
            let (linear_min, linear_max) = Self::linear_ext_min_max(last_point, coord_max, args);
            max = max.max(linear_min.max(linear_max));
        }

        for point in self.points.iter() {
            max = max.max(point.value.get_max(args));
        }

        for i in 0..(self.points.len() - 1) {
            let point = &self.points[i];
            let next_point = &self.points[i + 1];
            let distance = next_point.location - point.location;
            let point_max = point.value.get_max(args);
            let next_point_max = next_point.value.get_max(args);
            // Calculate new max if at least one derivative is non-zero
            if point.derivative != 0.0 || next_point.derivative != 0.0 {
                let point_deriv_scaled = point.derivative * distance;
                let val1 = point_deriv_scaled - next_point.value.get_min(args) + point_max;
                let next_point_deriv_scaled = next_point.derivative * distance;
                let val2 = -next_point_deriv_scaled + next_point_max - point.value.get_min(args);
                let highest = val1.max(val2);
                let points_max = point_max.max(next_point_max);
                max = max.max(points_max + (highest * 0.25));
            }
        }
        max
    }

    fn get_tree_hash(&self, state: &mut AHasher) {
        self.coordinate.get_tree_hash(state);
        for p in self.points.iter() {
            p.derivative.to_be_bytes().hash(state);
            p.location.to_be_bytes().hash(state);
            p.value.get_tree_hash(state);
        }
    }

    fn precompute_noise_instance(&self, dimension: &str) {
        self.coordinate.precompute_noise_instance(dimension);
        for p in self.points.iter() {
            p.value.precompute_noise_instance(dimension);
        }
    }

    fn get_max_branch_depth(&self, outline: &mut DensityFnOutline) -> u16 {
        let mut max_depth = outline.selected_frame.borrow().largest_slot as u16;
        for (i, p) in self.points.iter().enumerate() {
            max_depth = max_depth.max(p.value.get_max_branch_depth(outline));
        }
        max_depth
    }

    fn generate_state(&self, dimension: &str, outline: &mut DensityFnOutline) {
        // Push spline primary end frame
        let frame_pos = outline.stack.len();
        outline.new_stack_frame(DensityFnOutlineType::Spline);
        outline.constant_args.push(2.0);

        // Generate spline points
        let num_of_points = self.points.len();
        let init_stack_location = outline.stack.len();
        let mut point_constants = vec![0.0; num_of_points * 3];
        let mut point_stack_positions = vec![0; num_of_points];
        let mut stack_const_indexes = vec![0; num_of_points];
        for (i, p) in self.points.iter().enumerate() {
            point_stack_positions[i] = outline.stack.len();
            outline.new_stack_frame(DensityFnOutlineType::Spline);
            outline.constant_args.push(1.0); // Spline point value indicator constant
            stack_const_indexes[i] = outline.constant_args.len();
            outline.constant_args.push(0.0);

            point_constants[i] = p.location;
            point_constants[i + num_of_points] = p.derivative;
            //outline.constant_args.push(p.derivative);
            //outline.constant_args.push(p.location);
            p.value.generate_state(dimension, outline);
            point_constants[i + (num_of_points * 2)] = outline.stack.len() as f64;
        }

        // Generate spline primary
        //let reg_pos = self.get_max_branch_depth(outline) as u8;
        let primary_frame_pos = outline.stack.len();
        outline.selected_frame.borrow_mut().largest_slot += 1;
        outline.new_stack_frame(DensityFnOutlineType::Spline);
        //outline.selected_frame.borrow_mut().largest_slot += 1;
        //outline.new_stack_frame(DensityFnOutlineType::Spline);
        let stack_pos = outline.stack.len() - 1;
        for index in stack_const_indexes {
            outline.constant_args[index] = stack_pos as f64;
        }
        //let spline_reg = outline.stack[stack_pos].reg_position as u16;
        for (i, stack_pos) in point_stack_positions.iter().enumerate() {
            let spline_reg = outline
                .stack
                .get_mut(point_constants[i + (num_of_points * 2)] as usize - 1)
                .unwrap()
                .reg_position;
            let point_frame = outline.stack.get_mut(*stack_pos).unwrap();
            if let SplineArgsType::Args(_) = self.points[i].value {
                point_frame.arg_types[0] = DensityOutlineArgType::Function as u8;
                point_frame.arg_positions[0] = spline_reg as u16;
            }
        }
        outline.constant_args.push(0.0); // Spline primary indicator
        outline.constant_args.push(num_of_points as f64); // points count
        outline.constant_args.push(init_stack_location as f64); // used when spline finished
        outline.constant_args.extend(point_constants);
        outline.selected_frame.borrow_mut().largest_slot += 1;
        self.coordinate.0.generate_state(dimension, outline);

        // Update spline end frame with spline start frame data
        let reg_pos = outline.stack[primary_frame_pos].reg_position;
        outline.stack[frame_pos].arg_types[0] = DensityOutlineArgType::Function as u8;
        outline.stack[frame_pos].arg_positions[0] = reg_pos as u16;
    }

    #[inline(always)]
    fn linear_ext_min_max(point: &SplinePoint, x: f64, args: &mut DensityFnArgs) -> (f64, f64) {
        let linear_min = Self::linear_ext_if_non_zero(point, x, point.value.get_min(args));
        let linear_max = Self::linear_ext_if_non_zero(point, x, point.value.get_max(args));
        (linear_min, linear_max)
    }

    #[inline(always)]
    fn linear_ext_if_non_zero(point: &SplinePoint, x: f64, value: f64) -> f64 {
        if point.derivative == 0.0 {
            value
        } else {
            value + (point.derivative * (x - point.location))
        }
    }
}

#[derive(Deserialize, Debug)]
struct SplineCoordinate(DensityArg);

impl SplineCoordinate {
    #[inline]
    fn compute(&self, args: &mut DensityFnArgs) -> f64 {
        self.0.compute(args)
    }

    #[inline]
    fn get_min(&self, args: &mut DensityFnArgs) -> f64 {
        self.0.get_min(args)
    }

    #[inline]
    fn get_max(&self, args: &mut DensityFnArgs) -> f64 {
        self.0.get_max(args)
    }

    fn get_tree_hash(&self, state: &mut AHasher) {
        self.0.get_tree_hash(state);
    }

    fn precompute_noise_instance(&self, dimension: &str) {
        self.0.precompute_noise_instance(dimension);
    }
}

#[derive(Deserialize, Debug)]
struct SplinePoint {
    derivative: f64,
    location: f64,
    value: SplineArgsType,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum SplineArgsType {
    Constant(f64),
    Args(SplineArgs),
}

impl SplineArgsType {
    #[inline]
    fn compute(&self, args: &mut DensityFnArgs) -> f64 {
        match self {
            SplineArgsType::Constant(value) => *value,
            SplineArgsType::Args(spline_args) => spline_args.compute(args),
        }
    }

    #[inline]
    fn get_min(&self, args: &mut DensityFnArgs) -> f64 {
        match self {
            SplineArgsType::Constant(value) => *value,
            SplineArgsType::Args(spline_args) => spline_args.get_min(args),
        }
    }

    #[inline]
    fn get_max(&self, args: &mut DensityFnArgs) -> f64 {
        match self {
            SplineArgsType::Constant(value) => *value,
            SplineArgsType::Args(spline_args) => spline_args.get_max(args),
        }
    }

    fn get_tree_hash(&self, state: &mut AHasher) {
        match self {
            SplineArgsType::Constant(value) => (*value).to_be_bytes().hash(state),
            SplineArgsType::Args(spline_args) => spline_args.get_tree_hash(state),
        }
    }

    fn precompute_noise_instance(&self, dimension: &str) {
        match self {
            SplineArgsType::Args(spline_args) => spline_args.precompute_noise_instance(dimension),
            _ => {}
        }
    }

    fn get_max_branch_depth(&self, outline: &mut DensityFnOutline) -> u16 {
        match self {
            SplineArgsType::Constant(_) => 0,
            SplineArgsType::Args(spline_args) => spline_args.get_max_branch_depth(outline) + 1,
        }
    }

    fn generate_state(&self, dimension: &str, outline: &mut DensityFnOutline) {
        match self {
            SplineArgsType::Constant(value) => {
                outline.set_stack_arg(DensityOutlineArgType::Constant);
                outline.constant_args.push(*value);
                //outline.selected_frame.borrow_mut().arg_num += 1;
            }
            SplineArgsType::Args(spline_args) => {
                spline_args.generate_state(dimension, outline);
            }
        }
    }
}
