/**
*   Testing for gpu-accelerated terrain gen through OpenCL
*/
use std::ffi::c_void;

use once_cell::sync::Lazy;
use opencl3::{
    command_queue::CommandQueue,
    context::Context,
    device::{get_all_devices, Device, CL_DEVICE_TYPE_GPU},
    kernel::Kernel,
    memory::{Buffer, CL_MEM_COPY_HOST_PTR, CL_MEM_READ_ONLY, CL_MEM_WRITE_ONLY},
    program::Program,
    types::CL_BLOCKING,
};

use crate::server::terrain_gen::{
    func_deserialize::{DensityFnOutline, FunctionStackFrame},
    noise_generator::{ImprovedNoise, VanillaNoise, VanillaNoiseState},
};

pub static CL_CONTEXT: Lazy<Context> = Lazy::new(|| {
    println!("[Server] Creating GPU Device Context...");
    let time = std::time::Instant::now();
    let device = get_all_devices(CL_DEVICE_TYPE_GPU).unwrap().remove(0);
    let context = Context::from_device(&Device::new(device)).unwrap();
    println!(
        "[Server] Finished Creating GPU Context ({:?})",
        time.elapsed()
    );
    context
});

pub static CL_QUEUE: Lazy<CommandQueue> = Lazy::new(|| {
    println!("[Server] Creating GPU Command Queue...");
    let time = std::time::Instant::now();
    let queue = CommandQueue::create_default(&CL_CONTEXT, 0).unwrap();
    println!("[Server] Finished Creating Queue ({:?})", time.elapsed());
    queue
});

pub static CL_PROGRAM_ADD: Lazy<Program> = Lazy::new(|| {
    let source = std::fs::read_to_string("assets/opencl/add.cl").unwrap();
    Program::create_and_build_from_source(&CL_CONTEXT, &source, "-I assets/opencl").unwrap()
});

pub static CL_PROGRAM_EVAL: Lazy<Program> = Lazy::new(|| {
    println!("[Server] Compiling OpenCL Compute Kernel for Chunk Generation...");
    let time = std::time::Instant::now();
    let source = std::fs::read_to_string("assets/opencl/eval.cl").unwrap();
    // IMPORTANT: -cl-fast-relaxed-math compilation option changes the accuracy of math operations, specifically the
    // noise generator, but results in slightly faster performance. Generally advised to leave
    // disabled
    let program = Program::create_and_build_from_source(&CL_CONTEXT, &source, "-cl-std=CL1.2 -I assets\\opencl -I assets\\opencl\\functions -cl-mad-enable -cl-finite-math-only -cl-unsafe-math-optimizations").unwrap();
    println!(
        "[Server] Finished Compiling Chunk Generation Compute Kernel ({:?})",
        time.elapsed()
    );
    program
});

pub fn compute_density_function(state: DensityFnOutline) -> Vec<f64> {
    let data_size = 1; //5_070_000;
    let mut results = vec![0.0f64; data_size];
    let mut stack = state.stack;
    //let mut flow = state.function_flow;
    //let mut arg_types = state.flow_arg_types;
    let mut const_args = state.constant_args;
    let mut noise_states = state.noise_states;
    let mut noise_levels = state.noise_levels;
    let mut noise_amplitudes = state.noise_amplitudes;
    let stack_ptr = stack.as_mut_ptr() as *mut c_void;
    /*let flow_ptr = flow.as_mut_ptr() as *mut c_void;
    let arg_types_ptr = arg_types.as_mut_ptr() as *mut c_void;
    let const_args_ptr = const_args.as_mut_ptr() as *mut c_void;*/
    let noise_states_ptr = noise_states.as_mut_ptr() as *mut c_void;
    let levels_ptr = noise_levels.as_mut_ptr() as *mut c_void;
    let ampl_ptr = noise_amplitudes.as_mut_ptr() as *mut c_void;

    unsafe {
        let stack_buf = Buffer::<FunctionStackFrame>::create(
            &CL_CONTEXT,
            CL_MEM_READ_ONLY | CL_MEM_COPY_HOST_PTR,
            stack.len(),
            stack_ptr,
        )
        .unwrap();
        /*let mut flow_buf = Buffer::<DensityFnOutlineType>::create(
            &CL_CONTEXT,
            CL_MEM_READ_ONLY,
            flow.len(),
            std::ptr::null_mut(),
        )
        .unwrap();
        CL_QUEUE
            .enqueue_write_buffer(&mut flow_buf, CL_BLOCKING, 0, &flow, &[])
            .unwrap();
        let mut arg_types_buf = Buffer::<DensityOutlineArgType>::create(
            &CL_CONTEXT,
            CL_MEM_READ_ONLY,
            arg_types.len(),
            std::ptr::null_mut(),
        )
        .unwrap();
        CL_QUEUE
            .enqueue_write_buffer(&mut arg_types_buf, CL_BLOCKING, 0, &arg_types, &[])
            .unwrap();*/
        let mut const_args_buf = Buffer::<f64>::create(
            &CL_CONTEXT,
            CL_MEM_READ_ONLY,
            const_args.len(),
            std::ptr::null_mut(),
        )
        .unwrap();
        CL_QUEUE
            .enqueue_write_buffer(&mut const_args_buf, CL_BLOCKING, 0, &const_args, &[])
            .unwrap();
        let mut noise_states_buf = Buffer::<VanillaNoiseState>::create(
            &CL_CONTEXT,
            CL_MEM_READ_ONLY | CL_MEM_COPY_HOST_PTR,
            noise_states.len(),
            noise_states_ptr,
        )
        .unwrap();
        /*CL_QUEUE
        .enqueue_write_buffer(&mut noise_states_buf, CL_BLOCKING, 0, &noise_states, &[])
        .unwrap();*/
        let mut noise_levels_buf = Buffer::<ImprovedNoise>::create(
            &CL_CONTEXT,
            CL_MEM_READ_ONLY | CL_MEM_COPY_HOST_PTR,
            noise_levels.len(),
            levels_ptr,
        )
        .unwrap();
        /*CL_QUEUE
        .enqueue_write_buffer(&mut noise_levels_buf, CL_BLOCKING, 0, &noise_levels, &[])
        .unwrap();*/
        let mut noise_amplitudes_buf = Buffer::<f64>::create(
            &CL_CONTEXT,
            CL_MEM_READ_ONLY | CL_MEM_COPY_HOST_PTR,
            noise_amplitudes.len(),
            ampl_ptr,
        )
        .unwrap();
        /*CL_QUEUE
        .enqueue_write_buffer(
            &mut noise_amplitudes_buf,
            CL_BLOCKING,
            0,
            &noise_amplitudes,
            &[],
        )
        .unwrap();*/
        let out_buf = Buffer::<f64>::create(
            &CL_CONTEXT,
            CL_MEM_WRITE_ONLY,
            data_size,
            std::ptr::null_mut(),
        )
        .unwrap();

        let kernel = Kernel::create(&CL_PROGRAM_EVAL, "eval").unwrap();
        kernel.set_arg(0, &stack_buf).unwrap();
        kernel.set_arg(1, &(stack.len() as i16 - 1)).unwrap();
        //kernel.set_arg(0, &flow_buf).unwrap();
        //kernel.set_arg(1, &arg_types_buf).unwrap();
        kernel.set_arg(2, &const_args_buf).unwrap();
        kernel.set_arg(3, &noise_states_buf).unwrap();
        kernel.set_arg(4, &noise_levels_buf).unwrap();
        kernel.set_arg(5, &noise_amplitudes_buf).unwrap();
        kernel.set_arg(6, &out_buf).unwrap();

        CL_QUEUE
            .enqueue_nd_range_kernel(
                kernel.get(),
                1,
                std::ptr::null_mut(),
                [data_size].as_ptr(),
                std::ptr::null_mut(),
                &[],
            )
            .unwrap();

        CL_QUEUE
            .enqueue_read_buffer(&out_buf, CL_BLOCKING, 0, &mut results, &[])
            .unwrap();
    }
    results
}

pub fn get_noise(noise_generator: VanillaNoise, positions: Vec<u64>) -> Vec<f64> {
    let data_size = positions.len();
    let mut result = vec![0.0f64; data_size];
    let mut noise_state = vec![noise_generator.get_state(0)];
    let mut noise_levels = noise_generator.get_all_levels();
    let mut noise_ampl = noise_generator.get_all_amplitudes();
    let state_ptr = noise_state.as_mut_ptr() as *mut c_void;
    let levels_ptr = noise_levels.as_mut_ptr() as *mut c_void;
    let ampl_ptr = noise_ampl.as_mut_ptr() as *mut c_void;

    unsafe {
        //let mut pos_buf = Buffer::<u64>::create(&CL_CONTEXT, CL_MEM_READ_ONLY, data_size, std::ptr::null_mut()).unwrap();
        let state_buf = Buffer::<VanillaNoiseState>::create(
            &CL_CONTEXT,
            CL_MEM_READ_ONLY | CL_MEM_COPY_HOST_PTR,
            noise_state.len(),
            state_ptr,
        )
        .unwrap();
        let levels_buf = Buffer::<ImprovedNoise>::create(
            &CL_CONTEXT,
            CL_MEM_READ_ONLY | CL_MEM_COPY_HOST_PTR,
            noise_levels.len(),
            levels_ptr,
        )
        .unwrap();
        let ampl_buf = Buffer::<f64>::create(
            &CL_CONTEXT,
            CL_MEM_READ_ONLY | CL_MEM_COPY_HOST_PTR,
            noise_ampl.len(),
            ampl_ptr,
        )
        .unwrap();
        let res_buf = Buffer::<f64>::create(
            &CL_CONTEXT,
            CL_MEM_WRITE_ONLY,
            data_size,
            std::ptr::null_mut(),
        )
        .unwrap();

        //CL_QUEUE.enqueue_write_buffer(&mut pos_buf, CL_BLOCKING, 0, &positions, &[]).unwrap();

        let kernel = Kernel::create(&CL_PROGRAM_ADD, "add").unwrap();
        //kernel.set_arg(0, &pos_buf).unwrap();
        kernel.set_arg(0, &state_buf).unwrap();
        kernel.set_arg(1, &levels_buf).unwrap();
        kernel.set_arg(2, &ampl_buf).unwrap();
        kernel.set_arg(3, &res_buf).unwrap();

        CL_QUEUE
            .enqueue_nd_range_kernel(
                kernel.get(),
                1,
                std::ptr::null_mut(),
                [data_size].as_ptr(),
                std::ptr::null_mut(),
                &[],
            )
            .unwrap();

        CL_QUEUE
            .enqueue_read_buffer(&res_buf, CL_BLOCKING, 0, &mut result, &[])
            .unwrap();
    }
    result
}

pub fn add_vecs() -> Vec<f64> {
    let data_size = 4_096;
    let a_vec = vec![2.0f64; data_size];
    let b_vec = vec![6.0f64; data_size];
    let mut result = vec![0.0f64; data_size];

    //println!("{:?}", device);

    unsafe {
        let mut buf_a = Buffer::<f64>::create(
            &CL_CONTEXT,
            CL_MEM_READ_ONLY,
            data_size,
            std::ptr::null_mut(),
        )
        .unwrap();
        let mut buf_b = Buffer::<f64>::create(
            &CL_CONTEXT,
            CL_MEM_READ_ONLY,
            data_size,
            std::ptr::null_mut(),
        )
        .unwrap();
        let buf_res = Buffer::<f64>::create(
            &CL_CONTEXT,
            CL_MEM_WRITE_ONLY,
            data_size,
            std::ptr::null_mut(),
        )
        .unwrap();
        CL_QUEUE
            .enqueue_write_buffer(&mut buf_a, CL_BLOCKING, 0, &a_vec, &[])
            .unwrap();
        CL_QUEUE
            .enqueue_write_buffer(&mut buf_b, CL_BLOCKING, 0, &b_vec, &[])
            .unwrap();

        let kernel = Kernel::create(&CL_PROGRAM_ADD, "add").unwrap();
        kernel.set_arg(0, &buf_a).unwrap();
        kernel.set_arg(1, &buf_b).unwrap();
        kernel.set_arg(2, &buf_res).unwrap();

        CL_QUEUE
            .enqueue_nd_range_kernel(
                kernel.get(),
                3,
                std::ptr::null_mut(),
                [16, 16, 16].as_ptr(),
                std::ptr::null_mut(),
                &[],
            )
            .unwrap();

        CL_QUEUE
            .enqueue_read_buffer(&buf_res, CL_BLOCKING, 0, &mut result, &[])
            .unwrap();
    }

    result
}

pub fn add_vecs_seq() -> Vec<f64> {
    let data_size = 1_200;
    let a_vec = vec![2.0f64; data_size];
    let b_vec = vec![6.0f64; data_size];
    let mut results = vec![0.0f64; data_size];

    for (i, val) in results.iter_mut().enumerate() {
        let a = a_vec[i];
        let b = b_vec[i];
        let mut r = 0.0;
        for _ in 0..1_000 {
            r += a * b + i as f64;
        }
        *val = r + 8.0;
    }
    results
}
