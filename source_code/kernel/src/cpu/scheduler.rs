// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Cooperative scheduler scaffolding for multi-core bring-up experiments.

use core::arch::asm;

/// Constantes para configuración del scheduler y el timer
pub const MAX_CORES: usize = 4;
pub const MAX_KERNEL_TASKS: usize = 2;
pub const MAX_USER_TASKS: usize = 3;

/// Índice de tarea actual por core
#[no_mangle]
pub static mut CURRENT_TASK_IDX: [usize; MAX_CORES] = [0; MAX_CORES];

/// Tabla de tareas del kernel
#[no_mangle]
pub static mut KERNEL_TASKS: [unsafe extern "C" fn(); MAX_KERNEL_TASKS] = [
    kernel_task0,
    kernel_task1,
];

/// Tabla de tareas de usuario/driver
#[no_mangle]
pub static mut USER_TASKS: [unsafe extern "C" fn(); MAX_USER_TASKS] = [
    user_task0,
    user_task1,
    user_task2,
];

/// Plantillas/ejemplos de tareas del kernel
#[no_mangle]
pub unsafe extern "C" fn kernel_task0() {
    // Código de la tarea de kernel 0
}

#[no_mangle]
pub unsafe extern "C" fn kernel_task1() {
    // Código de la tarea de kernel 1
}

/// Plantillas/ejemplos de tareas de usuario/driver
#[no_mangle]
pub unsafe extern "C" fn user_task0() {
    // Código de la tarea user/driver 0
}

#[no_mangle]
pub unsafe extern "C" fn user_task1() {
    // Código de la tarea user/driver 1
}

#[no_mangle]
pub unsafe extern "C" fn user_task2() {
    // Código de la tarea user/driver 2
}

/// Tabla vectorial ARM64 (Exception Vector Table), alineada a 2 KiB.
/// Esta tabla define a dónde saltar cuando ocurre una excepción/interrupción.
#[repr(align(2048))]
#[no_mangle]
pub static VECTOR_TABLE: [u32; 512] = [
    // 0x000: Synchronous EL1t
    0x14000000, // b .
    0; 31,

    // 0x020: IRQ EL1t
    0x14000000, // b .
    0; 31,

    // 0x040: FIQ EL1t
    0x14000000, // b .
    0; 31,

    // 0x060: SError EL1t
    0x14000000, // b .
    0; 31,

    // 0x080: Synchronous EL1h
    0x14000000, // b .
    0; 31,

    // 0x0A0: IRQ EL1h (entrada principal de IRQ en modo kernel)
    // NOTA: Al instalar la tabla se debe escribir aquí la instrucción bl scheduler_irq_handler
    0x94000000, // bl (offset a parchear en tiempo de link o con binpatch)
    0; 31,

    // 0x0C0: FIQ EL1h
    0x14000000, // b .
    0; 31,

    // 0x0E0: SError EL1h
    0x14000000, // b .
    0; 31,
];

/// Instala la tabla vectorial: el CPU la usará para saltar a los handlers de excepción/IRQ.
/// Llamar durante la inicialización del kernel.
pub unsafe fn install_vector_table() {
    let addr = &VECTOR_TABLE as *const _ as usize;
    asm!(
        "msr vbar_el1, {0}",
        in(reg) addr,
        options(nostack, preserves_flags)
    );
}

/// Habilita las IRQs a nivel de CPU (DAIF register).
pub unsafe fn enable_irq() {
    asm!(
        "msr daifclr, #2",
        options(nostack, preserves_flags)
    );
}

/// Configura el Generic Timer del CPU para que lance una IRQ cada 5 milisegundos.
/// Debe llamarse después de configurar la vector table y antes de entrar en main loop.
///
/// - Lee la frecuencia del timer (`cntfrq_el0`), que normalmente es fija (e.g. 50MHz).
/// - Calcula los ticks para 5ms.
/// - Escribe el valor en `cntp_tval_el0` (timer del núcleo actual).
/// - Habilita el timer y su IRQ vía `cntp_ctl_el0`.
pub unsafe fn setup_generic_timer_5ms() {
    let mut freq: u32;
    let ticks: u64;

    // Lee la frecuencia del timer (Hz) del registro cntfrq_el0
    asm!(
        "mrs {freq}, cntfrq_el0",
        freq = out(reg) freq
    );
    // Calcula los ticks para 5 ms: ticks = freq * 0.005
    ticks = (freq as u64) / 200;

    // Configura el timer del núcleo: cuenta descendente, IRQ al llegar a 0
    asm!(
        "msr cntp_tval_el0, {ticks}", // Valor inicial del timer
        "mov x0, #1",
        "msr cntp_ctl_el0, x0",       // Habilita timer y su IRQ
        ticks = in(reg) ticks,
        out("x0") _
    );
}

/// Handler del scheduler, llamado por la IRQ del timer cada 5ms.
/// Alterna tareas según el núcleo, llamando a funciones kernel en core 0,
/// y a funciones de usuario/driver en el resto.
#[no_mangle]
pub unsafe extern "C" fn scheduler_irq_handler() {
    asm!(
        // Reprograma el timer para el próximo tick de 5ms (misma frecuencia)
        "mrs x10, cntfrq_el0",
        "udiv x10, x10, #200",            // x10 = ticks para 5ms
        "msr cntp_tval_el0, x10",

        // Guarda x0, x1 (preserva valores originales)
        "stp x0, x1, [sp, #-16]!",

        // 1. Lee el número de núcleo actual (Affinity Level 0)
        "mrs x0, mpidr_el1",
        "and x0, x0, #0b11",              // x0 = core_id (0..3)

        // 2. Obtiene puntero a CURRENT_TASK_IDX
        "ldr x1, ={current_task_idx}",

        // 3. Calcula dirección del índice para este core
        "add x2, x1, x0, lsl #3",         // x2 = &CURRENT_TASK_IDX[core_id]

        // 4. Carga el índice de tarea actual de este core
        "ldr x3, [x2]",                   // x3 = idx_tarea_actual

        // 5. Suma 1 al índice (próxima tarea)
        "add x3, x3, #1",

        // 6. Chequea el máximo de tareas para el core
        "cmp x0, #0",
        "b.eq 1f",
        // Núcleos 1,2,3: User/driver
        "mov x4, {max_user_tasks}",
        "cmp x3, x4",
        "csel x3, xzr, x3, eq",
        "b 2f",
        // Núcleo 0: Kernel
        "1:",
        "mov x4, {max_kernel_tasks}",
        "cmp x3, x4",
        "csel x3, xzr, x3, eq",
        "2:",
        // 7. Guarda el nuevo índice de tarea para este core
        "str x3, [x2]",

        // 8. Elige tabla y llama a la función correspondiente
        "cmp x0, #0",
        "b.eq 3f",
        // User/driver
        "ldr x5, ={user_tasks}",
        "ldr x6, [x2]",
        "ldr x7, [x5, x6, lsl #3]",
        "blr x7",
        "b 4f",
        // Kernel
        "3:",
        "ldr x5, ={kernel_tasks}",
        "ldr x6, [x2]",
        "ldr x7, [x5, x6, lsl #3]",
        "blr x7",
        "4:",
        // Restaura x0, x1
        "ldp x0, x1, [sp], #16",
        "eret",
        current_task_idx = sym CURRENT_TASK_IDX,
        kernel_tasks = sym KERNEL_TASKS,
        user_tasks = sym USER_TASKS,
        max_kernel_tasks = const MAX_KERNEL_TASKS,
        max_user_tasks = const MAX_USER_TASKS,
        options(noreturn)
    );
}

// === Ejemplo de inicialización básica en main o el entrypoint de tu kernel ===
// unsafe {
//     install_vector_table();       // Instala la tabla vectorial (handlers de excepción)
//     setup_generic_timer_5ms();    // Configura el timer para lanzar IRQ cada 5 ms
//     enable_irq();                 // Habilita IRQs
// }
