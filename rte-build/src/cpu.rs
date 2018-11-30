use std::iter;

use raw_cpuid;

pub fn gen_cpu_features() -> impl Iterator<Item = String> {
    let mut cflags = vec![];
    let mut compile_time_cpuflags = vec![];

    let cpuid = raw_cpuid::CpuId::new();

    if let Some(features) = cpuid.get_feature_info() {
        if features.has_sse() {
            cflags.push("-DRTE_MACHINE_CPUFLAG_SSE");
            compile_time_cpuflags.push("RTE_CPUFLAG_SSE");
        }
        if features.has_sse2() {
            cflags.push("-DRTE_MACHINE_CPUFLAG_SSE2");
            compile_time_cpuflags.push("RTE_CPUFLAG_SSE2");
        }
        if features.has_sse3() {
            cflags.push("-DRTE_MACHINE_CPUFLAG_SSE3");
            compile_time_cpuflags.push("RTE_CPUFLAG_SSE3");
        }
        if features.has_ssse3() {
            cflags.push("-DRTE_MACHINE_CPUFLAG_SSSE3");
            compile_time_cpuflags.push("RTE_CPUFLAG_SSSE3");
        }
        if features.has_sse41() {
            cflags.push("-DRTE_MACHINE_CPUFLAG_SSE4_1");
            compile_time_cpuflags.push("RTE_CPUFLAG_SSE4_1");
        }
        if features.has_sse42() {
            cflags.push("-DRTE_MACHINE_CPUFLAG_SSE4_2");
            compile_time_cpuflags.push("RTE_CPUFLAG_SSE4_2");
        }
        if features.has_aesni() {
            cflags.push("-DRTE_MACHINE_CPUFLAG_AES");
            compile_time_cpuflags.push("RTE_CPUFLAG_AES");
        }
        if features.has_pclmulqdq() {
            cflags.push("-DRTE_MACHINE_CPUFLAG_PCLMULQDQ");
            compile_time_cpuflags.push("RTE_CPUFLAG_PCLMULQDQ");
        }
        if features.has_avx() {
            cflags.push("-DRTE_MACHINE_CPUFLAG_AVX");
            compile_time_cpuflags.push("RTE_CPUFLAG_AVX");
        }
        if features.has_rdrand() {
            cflags.push("-DRTE_MACHINE_CPUFLAG_RDRAND");
        }
        if features.has_f16c() {
            cflags.push("-DRTE_MACHINE_CPUFLAG_F16C");
        }
    }

    if let Some(features) = cpuid.get_extended_feature_info() {
        if features.has_fsgsbase() {
            cflags.push("-DRTE_MACHINE_CPUFLAG_FSGSBASE");
        }
        if features.has_avx2() {
            cflags.push("-DRTE_MACHINE_CPUFLAG_AVX2");
            compile_time_cpuflags.push("RTE_CPUFLAG_AVX2");
        }
        if features.has_avx512f() {
            cflags.push("-RTE_MACHINE_CPUFLAG_AVX512F");
            compile_time_cpuflags.push("RTE_CPUFLAG_AVX512F");
        }
    }

    cflags
        .into_iter()
        .map(|s| s.to_owned())
        .chain(iter::once(format!(
            "-DRTE_COMPILE_TIME_CPUFLAGS={}",
            itertools::join(compile_time_cpuflags, ",")
        )))
}
