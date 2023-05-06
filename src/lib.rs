use anyhow::Result;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{
        self, alpha1, alphanumeric1, line_ending, not_line_ending, one_of, space0,
    },
    combinator::{map, map_res, opt, recognize, value},
    multi::{many1, separated_list0, separated_list1},
    number::complete::float,
    sequence::{delimited, pair, preceded, separated_pair, terminated, tuple},
    IResult,
};

#[derive(Debug, PartialEq)]
pub struct AddressSizes {
    pub physical_size: u32,
    pub virtual_size: u32,
}

#[derive(Debug)]
pub struct CpuInfo<'a> {
    pub cpus: Vec<Cpu<'a>>,
}

#[derive(Debug)]
pub struct Cpu<'a> {
    pub processor: u32,
    pub vendor_id: &'a str,
    pub cpu_family: u32,
    pub model: u32,
    pub model_name: &'a str,
    pub stepping: u32,
    pub microcode: u32,
    pub cpu_mhz: f32,
    pub cache_size: u32,
    pub physical_id: u32,
    pub siblings: u32,
    pub core_id: u32,
    pub cpu_cores: u32,
    pub apicid: u32,
    pub initial_apicid: u32,
    pub fpu: bool,
    pub fpu_exception: bool,
    pub cpuid_level: u32,
    pub wp: bool,
    pub flags: Vec<&'a str>,
    pub vmx_flags: Vec<&'a str>,
    pub bugs: Vec<&'a str>,
    pub bogomips: f32,
    pub clflush_size: u32,
    pub cache_alignment: u32,
    pub address_sizes: AddressSizes,
    pub power_management: Option<&'a str>,
}

pub fn cpuinfo(input: &'static str) -> Result<CpuInfo> {
    let (_, cpus) = cpus(input)?;
    Ok(CpuInfo { cpus })
}

fn separator(input: &str) -> IResult<&str, ()> {
    value((), delimited(space0, tag(":"), space0))(input)
}

fn field_value<'a, F, V, T>(
    field_name: F,
    field_value: V,
) -> impl FnMut(&'a str) -> IResult<&'a str, T>
where
    F: FnMut(&'a str) -> IResult<&'a str, &str>,
    V: FnMut(&'a str) -> IResult<&'a str, T>,
{
    map(
        terminated(
            separated_pair(field_name, separator, field_value),
            line_ending,
        ),
        |(_, v)| v,
    )
}

fn boolean(input: &str) -> IResult<&str, bool> {
    map(alt((tag("yes"), tag("no"))), |v| match v {
        "yes" => true,
        "no" => false,
        _ => unreachable!(),
    })(input)
}

fn list(input: &str) -> IResult<&str, Vec<&str>> {
    separated_list0(
        tag(" "),
        recognize(many1(one_of("abcdefghijklmnopqrstuvwxyz01234567890_"))),
    )(input)
}

fn hexadecimal(input: &str) -> IResult<&str, u32> {
    map_res(
        preceded(
            alt((tag("0x"), tag("0X"))),
            recognize(many1(one_of("0123456789abcdefABCDEF"))),
        ),
        |out: &str| u32::from_str_radix(out, 16),
    )(input)
}

fn processor(input: &str) -> IResult<&str, u32> {
    field_value(tag("processor"), complete::u32)(input)
}

fn vendor_id(input: &str) -> IResult<&str, &str> {
    field_value(tag("vendor_id"), alpha1)(input)
}

fn cpu_family(input: &str) -> IResult<&str, u32> {
    field_value(tag("cpu family"), complete::u32)(input)
}

fn model(input: &str) -> IResult<&str, u32> {
    field_value(tag("model"), complete::u32)(input)
}

fn model_name(input: &str) -> IResult<&str, &str> {
    field_value(tag("model name"), not_line_ending)(input)
}

fn stepping(input: &str) -> IResult<&str, u32> {
    field_value(tag("stepping"), complete::u32)(input)
}

fn microcode(input: &str) -> IResult<&str, u32> {
    field_value(tag("microcode"), hexadecimal)(input)
}

fn cpu_mhz(input: &str) -> IResult<&str, f32> {
    field_value(tag("cpu MHz"), float)(input)
}

fn cache_size(input: &str) -> IResult<&str, u32> {
    map(
        terminated(
            separated_pair(tag("cache size"), separator, complete::u32),
            tuple((space0, tag("KB"), line_ending)),
        ),
        |(_, cache_size)| cache_size * 1024,
    )(input)
}

fn physical_id(input: &str) -> IResult<&str, u32> {
    field_value(tag("physical id"), complete::u32)(input)
}

fn siblings(input: &str) -> IResult<&str, u32> {
    field_value(tag("siblings"), complete::u32)(input)
}

fn core_id(input: &str) -> IResult<&str, u32> {
    field_value(tag("core id"), complete::u32)(input)
}

fn cpu_cores(input: &str) -> IResult<&str, u32> {
    field_value(tag("cpu cores"), complete::u32)(input)
}

fn apicid(input: &str) -> IResult<&str, u32> {
    field_value(tag("apicid"), complete::u32)(input)
}

fn initial_apicid(input: &str) -> IResult<&str, u32> {
    field_value(tag("initial apicid"), complete::u32)(input)
}

fn fpu(input: &str) -> IResult<&str, bool> {
    field_value(tag("fpu"), boolean)(input)
}

fn fpu_exception(input: &str) -> IResult<&str, bool> {
    field_value(tag("fpu_exception"), boolean)(input)
}

fn cpuid_level(input: &str) -> IResult<&str, u32> {
    field_value(tag("cpuid level"), complete::u32)(input)
}

fn wp(input: &str) -> IResult<&str, bool> {
    field_value(tag("wp"), boolean)(input)
}

fn flags(input: &str) -> IResult<&str, Vec<&str>> {
    field_value(tag("flags"), list)(input)
}

fn vmx_flags(input: &str) -> IResult<&str, Vec<&str>> {
    field_value(tag("vmx flags"), list)(input)
}

fn bugs(input: &str) -> IResult<&str, Vec<&str>> {
    field_value(tag("bugs"), list)(input)
}

fn bogomips(input: &str) -> IResult<&str, f32> {
    field_value(tag("bogomips"), float)(input)
}

fn clflush_size(input: &str) -> IResult<&str, u32> {
    field_value(tag("clflush size"), complete::u32)(input)
}

fn cache_alignment(input: &str) -> IResult<&str, u32> {
    field_value(tag("cache_alignment"), complete::u32)(input)
}

fn physical_size(input: &str) -> IResult<&str, u32> {
    map(pair(complete::u32, tag(" bits physical")), |(v, _)| v)(input)
}

fn virtual_size(input: &str) -> IResult<&str, u32> {
    map(pair(complete::u32, tag(" bits virtual")), |(v, _)| v)(input)
}

fn address_sizes(input: &str) -> IResult<&str, AddressSizes> {
    field_value(
        tag("address sizes"),
        map(
            separated_pair(physical_size, tag(", "), virtual_size),
            |(physical_size, virtual_size)| AddressSizes {
                physical_size,
                virtual_size,
            },
        ),
    )(input)
}

fn power_management(input: &str) -> IResult<&str, Option<&str>> {
    field_value(tag("power management"), opt(alphanumeric1))(input)
}

fn cpu(input: &str) -> IResult<&str, Cpu> {
    let (input, processor) = processor(input)?;
    let (input, vendor_id) = vendor_id(input)?;
    let (input, cpu_family) = cpu_family(input)?;
    let (input, model) = model(input)?;
    let (input, model_name) = model_name(input)?;
    let (input, stepping) = stepping(input)?;
    let (input, microcode) = microcode(input)?;
    let (input, cpu_mhz) = cpu_mhz(input)?;
    let (input, cache_size) = cache_size(input)?;
    let (input, physical_id) = physical_id(input)?;
    let (input, siblings) = siblings(input)?;
    let (input, core_id) = core_id(input)?;
    let (input, cpu_cores) = cpu_cores(input)?;
    let (input, apicid) = apicid(input)?;
    let (input, initial_apicid) = initial_apicid(input)?;
    let (input, fpu) = fpu(input)?;
    let (input, fpu_exception) = fpu_exception(input)?;
    let (input, cpuid_level) = cpuid_level(input)?;
    let (input, wp) = wp(input)?;
    let (input, flags) = flags(input)?;
    let (input, vmx_flags) = vmx_flags(input)?;
    let (input, bugs) = bugs(input)?;
    let (input, bogomips) = bogomips(input)?;
    let (input, clflush_size) = clflush_size(input)?;
    let (input, cache_alignment) = cache_alignment(input)?;
    let (input, address_sizes) = address_sizes(input)?;
    let (input, power_management) = power_management(input)?;

    let cpu = Cpu {
        processor,
        vendor_id,
        cpu_family,
        model,
        model_name,
        stepping,
        microcode,
        cpu_mhz,
        cache_size,
        physical_id,
        siblings,
        core_id,
        cpu_cores,
        apicid,
        initial_apicid,
        fpu,
        fpu_exception,
        cpuid_level,
        wp,
        flags,
        vmx_flags,
        bugs,
        bogomips,
        clflush_size,
        cache_alignment,
        address_sizes,
        power_management,
    };

    Ok((input, cpu))
}

fn cpus(input: &str) -> IResult<&str, Vec<Cpu>> {
    separated_list1(line_ending, cpu)(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_processor() {
        let result = processor(
            "processor	: 0
",
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap().1, 0);
    }

    #[test]
    fn parses_vendor_id() {
        let result = vendor_id(
            "vendor_id	: GenuineIntel
",
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap().1, "GenuineIntel");
    }

    #[test]
    fn parses_cpu_family() {
        let result = cpu_family(
            "cpu family	: 6
",
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap().1, 6);
    }

    #[test]
    fn parses_model() {
        let result = model(
            "model		: 94
",
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap().1, 94);
    }

    #[test]
    fn parses_model_name() {
        let result = model_name(
            "model name	: Intel(R) Core(TM) i7-6700K CPU @ 4.00GHz
",
        );
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().1,
            "Intel(R) Core(TM) i7-6700K CPU @ 4.00GHz"
        );
    }

    #[test]
    fn parses_stepping() {
        let result = stepping(
            "stepping	: 3
",
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap().1, 3);
    }

    #[test]
    fn parses_microcode() {
        let result = microcode(
            "microcode	: 0xf0
",
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap().1, 240);
    }

    #[test]
    fn parses_cpu_mhz() {
        let result = cpu_mhz(
            "cpu MHz		: 4000.000
",
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap().1, 4000.00);
    }

    #[test]
    fn parses_cache_size() {
        let result = cache_size(
            "cache size	: 8192 KB
",
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap().1, 8192 * 1024);
    }

    #[test]
    fn parses_physical_id() {
        let result = physical_id(
            "physical id	: 0
",
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap().1, 0);
    }

    #[test]
    fn parses_siblings() {
        let result = siblings(
            "siblings	: 8
",
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap().1, 8);
    }

    #[test]
    fn parses_core_id() {
        let result = core_id(
            "core id		: 2
",
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap().1, 2);
    }

    #[test]
    fn parses_cpu_cores() {
        let result = cpu_cores(
            "cpu cores	: 4
",
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap().1, 4);
    }

    #[test]
    fn parses_apicid() {
        let result = apicid(
            "apicid		: 5
",
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap().1, 5);
    }

    #[test]
    fn parses_initial_apicid() {
        let result = initial_apicid(
            "initial apicid	: 5
",
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap().1, 5);
    }

    #[test]
    fn parses_fpu() {
        let result = fpu("fpu		: yes
");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().1, true);
    }

    #[test]
    fn parses_fpu_exception() {
        let result = fpu_exception(
            "fpu_exception		: yes
",
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap().1, true);
    }

    #[test]
    fn parses_cpuid_level() {
        let result = cpuid_level(
            "cpuid level	: 22
",
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap().1, 22);
    }

    #[test]
    fn parses_wp() {
        let result = wp("wp		: no
");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().1, false);
    }

    #[test]
    fn parses_flags() {
        let result = flags(
	    "flags		: fpu vme de pse tsc msr pae mce cx8 apic sep mtrr pge mca cmov pat pse36 clflush dts acpi mmx fxsr sse sse2 ss ht tm pbe syscall nx pdpe1gb rdtscp lm constant_tsc art arch_perfmon pebs bts rep_good nopl xtopology nonstop_tsc cpuid aperfmperf pni pclmulqdq dtes64 monitor ds_cpl vmx est tm2 ssse3 sdbg fma cx16 xtpr pdcm pcid sse4_1 sse4_2 x2apic movbe popcnt tsc_deadline_timer aes xsave avx f16c rdrand lahf_lm abm 3dnowprefetch cpuid_fault invpcid_single pti ssbd ibrs ibpb stibp tpr_shadow vnmi flexpriority ept vpid ept_ad fsgsbase tsc_adjust bmi1 avx2 smep bmi2 erms invpcid mpx rdseed adx smap clflushopt intel_pt xsaveopt xsavec xgetbv1 xsaves dtherm ida arat pln pts hwp hwp_notify hwp_act_window hwp_epp md_clear flush_l1d arch_capabilities
"
	);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().1,
            vec![
                "fpu",
                "vme",
                "de",
                "pse",
                "tsc",
                "msr",
                "pae",
                "mce",
                "cx8",
                "apic",
                "sep",
                "mtrr",
                "pge",
                "mca",
                "cmov",
                "pat",
                "pse36",
                "clflush",
                "dts",
                "acpi",
                "mmx",
                "fxsr",
                "sse",
                "sse2",
                "ss",
                "ht",
                "tm",
                "pbe",
                "syscall",
                "nx",
                "pdpe1gb",
                "rdtscp",
                "lm",
                "constant_tsc",
                "art",
                "arch_perfmon",
                "pebs",
                "bts",
                "rep_good",
                "nopl",
                "xtopology",
                "nonstop_tsc",
                "cpuid",
                "aperfmperf",
                "pni",
                "pclmulqdq",
                "dtes64",
                "monitor",
                "ds_cpl",
                "vmx",
                "est",
                "tm2",
                "ssse3",
                "sdbg",
                "fma",
                "cx16",
                "xtpr",
                "pdcm",
                "pcid",
                "sse4_1",
                "sse4_2",
                "x2apic",
                "movbe",
                "popcnt",
                "tsc_deadline_timer",
                "aes",
                "xsave",
                "avx",
                "f16c",
                "rdrand",
                "lahf_lm",
                "abm",
                "3dnowprefetch",
                "cpuid_fault",
                "invpcid_single",
                "pti",
                "ssbd",
                "ibrs",
                "ibpb",
                "stibp",
                "tpr_shadow",
                "vnmi",
                "flexpriority",
                "ept",
                "vpid",
                "ept_ad",
                "fsgsbase",
                "tsc_adjust",
                "bmi1",
                "avx2",
                "smep",
                "bmi2",
                "erms",
                "invpcid",
                "mpx",
                "rdseed",
                "adx",
                "smap",
                "clflushopt",
                "intel_pt",
                "xsaveopt",
                "xsavec",
                "xgetbv1",
                "xsaves",
                "dtherm",
                "ida",
                "arat",
                "pln",
                "pts",
                "hwp",
                "hwp_notify",
                "hwp_act_window",
                "hwp_epp",
                "md_clear",
                "flush_l1d",
                "arch_capabilities",
            ]
        )
    }

    #[test]
    fn parses_vmx_flags() {
        let result = vmx_flags(
	    "vmx flags	: vnmi preemption_timer invvpid ept_x_only ept_ad ept_1gb flexpriority tsc_offset vtpr mtf vapic ept vpid unrestricted_guest ple shadow_vmcs pml
"
	);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().1,
            vec![
                "vnmi",
                "preemption_timer",
                "invvpid",
                "ept_x_only",
                "ept_ad",
                "ept_1gb",
                "flexpriority",
                "tsc_offset",
                "vtpr",
                "mtf",
                "vapic",
                "ept",
                "vpid",
                "unrestricted_guest",
                "ple",
                "shadow_vmcs",
                "pml",
            ]
        )
    }

    #[test]
    fn parses_bugs() {
        let result = bugs(
	    "bugs		: cpu_meltdown spectre_v1 spectre_v2 spec_store_bypass l1tf mds swapgs taa itlb_multihit srbds mmio_stale_data retbleed
"
	);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().1,
            vec![
                "cpu_meltdown",
                "spectre_v1",
                "spectre_v2",
                "spec_store_bypass",
                "l1tf",
                "mds",
                "swapgs",
                "taa",
                "itlb_multihit",
                "srbds",
                "mmio_stale_data",
                "retbleed",
            ]
        )
    }

    #[test]
    fn parses_bogomips() {
        let result = bogomips(
            "bogomips	: 8003.30
",
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap().1, 8003.3);
    }

    #[test]
    fn parses_clflush_size() {
        let result = clflush_size(
            "clflush size	: 64
",
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap().1, 64);
    }

    #[test]
    fn parses_cache_alignment() {
        let result = cache_alignment(
            "cache_alignment	: 64
",
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap().1, 64);
    }

    #[test]
    fn parses_address_sizes() {
        let result = address_sizes(
            "address sizes	: 39 bits physical, 48 bits virtual
",
        );
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().1,
            AddressSizes {
                physical_size: 39,
                virtual_size: 48,
            }
        )
    }

    #[test]
    fn parses_power_management() {
        let result = power_management(
            "power management:
",
        );
        assert!(result.is_ok());
        assert!(result.unwrap().1.is_none());
    }

    #[test]
    fn parses_cpu() {
        let result = cpu(
	    "processor	: 6
vendor_id	: GenuineIntel
cpu family	: 6
model		: 94
model name	: Intel(R) Core(TM) i7-6700K CPU @ 4.00GHz
stepping	: 3
microcode	: 0xf0
cpu MHz		: 800.004
cache size	: 8192 KB
physical id	: 0
siblings	: 8
core id		: 2
cpu cores	: 4
apicid		: 5
initial apicid	: 5
fpu		: yes
fpu_exception	: yes
cpuid level	: 22
wp		: yes
flags		: fpu vme de pse tsc msr pae mce cx8 apic sep mtrr pge mca cmov pat pse36 clflush dts acpi mmx fxsr sse sse2 ss ht tm pbe syscall nx pdpe1gb rdtscp lm constant_tsc art arch_perfmon pebs bts rep_good nopl xtopology nonstop_tsc cpuid aperfmperf pni pclmulqdq dtes64 monitor ds_cpl vmx est tm2 ssse3 sdbg fma cx16 xtpr pdcm pcid sse4_1 sse4_2 x2apic movbe popcnt tsc_deadline_timer aes xsave avx f16c rdrand lahf_lm abm 3dnowprefetch cpuid_fault invpcid_single pti ssbd ibrs ibpb stibp tpr_shadow vnmi flexpriority ept vpid ept_ad fsgsbase tsc_adjust bmi1 avx2 smep bmi2 erms invpcid mpx rdseed adx smap clflushopt intel_pt xsaveopt xsavec xgetbv1 xsaves dtherm ida arat pln pts hwp hwp_notify hwp_act_window hwp_epp md_clear flush_l1d arch_capabilities
vmx flags	: vnmi preemption_timer invvpid ept_x_only ept_ad ept_1gb flexpriority tsc_offset vtpr mtf vapic ept vpid unrestricted_guest ple shadow_vmcs pml
bugs		: cpu_meltdown spectre_v1 spectre_v2 spec_store_bypass l1tf mds swapgs taa itlb_multihit srbds mmio_stale_data retbleed
bogomips	: 8003.30
clflush size	: 64
cache_alignment	: 64
address sizes	: 39 bits physical, 48 bits virtual
power management:
"
	);
        assert!(result.is_ok());
    }

    #[test]
    fn parses_cpus() {
        let result = cpus(
	   "processor	: 0
vendor_id	: GenuineIntel
cpu family	: 6
model		: 94
model name	: Intel(R) Core(TM) i7-6700K CPU @ 4.00GHz
stepping	: 3
microcode	: 0xf0
cpu MHz		: 971.836
cache size	: 8192 KB
physical id	: 0
siblings	: 8
core id		: 0
cpu cores	: 4
apicid		: 0
initial apicid	: 0
fpu		: yes
fpu_exception	: yes
cpuid level	: 22
wp		: yes
flags		: fpu vme de pse tsc msr pae mce cx8 apic sep mtrr pge mca cmov pat pse36 clflush dts acpi mmx fxsr sse sse2 ss ht tm pbe syscall nx pdpe1gb rdtscp lm constant_tsc art arch_perfmon pebs bts rep_good nopl xtopology nonstop_tsc cpuid aperfmperf pni pclmulqdq dtes64 monitor ds_cpl vmx est tm2 ssse3 sdbg fma cx16 xtpr pdcm pcid sse4_1 sse4_2 x2apic movbe popcnt tsc_deadline_timer aes xsave avx f16c rdrand lahf_lm abm 3dnowprefetch cpuid_fault invpcid_single pti ssbd ibrs ibpb stibp tpr_shadow vnmi flexpriority ept vpid ept_ad fsgsbase tsc_adjust bmi1 avx2 smep bmi2 erms invpcid mpx rdseed adx smap clflushopt intel_pt xsaveopt xsavec xgetbv1 xsaves dtherm ida arat pln pts hwp hwp_notify hwp_act_window hwp_epp md_clear flush_l1d arch_capabilities
vmx flags	: vnmi preemption_timer invvpid ept_x_only ept_ad ept_1gb flexpriority tsc_offset vtpr mtf vapic ept vpid unrestricted_guest ple shadow_vmcs pml
bugs		: cpu_meltdown spectre_v1 spectre_v2 spec_store_bypass l1tf mds swapgs taa itlb_multihit srbds mmio_stale_data retbleed
bogomips	: 8003.30
clflush size	: 64
cache_alignment	: 64
address sizes	: 39 bits physical, 48 bits virtual
power management:

processor	: 1
vendor_id	: GenuineIntel
cpu family	: 6
model		: 94
model name	: Intel(R) Core(TM) i7-6700K CPU @ 4.00GHz
stepping	: 3
microcode	: 0xf0
cpu MHz		: 1406.086
cache size	: 8192 KB
physical id	: 0
siblings	: 8
core id		: 1
cpu cores	: 4
apicid		: 2
initial apicid	: 2
fpu		: yes
fpu_exception	: yes
cpuid level	: 22
wp		: yes
flags		: fpu vme de pse tsc msr pae mce cx8 apic sep mtrr pge mca cmov pat pse36 clflush dts acpi mmx fxsr sse sse2 ss ht tm pbe syscall nx pdpe1gb rdtscp lm constant_tsc art arch_perfmon pebs bts rep_good nopl xtopology nonstop_tsc cpuid aperfmperf pni pclmulqdq dtes64 monitor ds_cpl vmx est tm2 ssse3 sdbg fma cx16 xtpr pdcm pcid sse4_1 sse4_2 x2apic movbe popcnt tsc_deadline_timer aes xsave avx f16c rdrand lahf_lm abm 3dnowprefetch cpuid_fault invpcid_single pti ssbd ibrs ibpb stibp tpr_shadow vnmi flexpriority ept vpid ept_ad fsgsbase tsc_adjust bmi1 avx2 smep bmi2 erms invpcid mpx rdseed adx smap clflushopt intel_pt xsaveopt xsavec xgetbv1 xsaves dtherm ida arat pln pts hwp hwp_notify hwp_act_window hwp_epp md_clear flush_l1d arch_capabilities
vmx flags	: vnmi preemption_timer invvpid ept_x_only ept_ad ept_1gb flexpriority tsc_offset vtpr mtf vapic ept vpid unrestricted_guest ple shadow_vmcs pml
bugs		: cpu_meltdown spectre_v1 spectre_v2 spec_store_bypass l1tf mds swapgs taa itlb_multihit srbds mmio_stale_data retbleed
bogomips	: 8003.30
clflush size	: 64
cache_alignment	: 64
address sizes	: 39 bits physical, 48 bits virtual
power management:

processor	: 2
vendor_id	: GenuineIntel
cpu family	: 6
model		: 94
model name	: Intel(R) Core(TM) i7-6700K CPU @ 4.00GHz
stepping	: 3
microcode	: 0xf0
cpu MHz		: 807.534
cache size	: 8192 KB
physical id	: 0
siblings	: 8
core id		: 2
cpu cores	: 4
apicid		: 4
initial apicid	: 4
fpu		: yes
fpu_exception	: yes
cpuid level	: 22
wp		: yes
flags		: fpu vme de pse tsc msr pae mce cx8 apic sep mtrr pge mca cmov pat pse36 clflush dts acpi mmx fxsr sse sse2 ss ht tm pbe syscall nx pdpe1gb rdtscp lm constant_tsc art arch_perfmon pebs bts rep_good nopl xtopology nonstop_tsc cpuid aperfmperf pni pclmulqdq dtes64 monitor ds_cpl vmx est tm2 ssse3 sdbg fma cx16 xtpr pdcm pcid sse4_1 sse4_2 x2apic movbe popcnt tsc_deadline_timer aes xsave avx f16c rdrand lahf_lm abm 3dnowprefetch cpuid_fault invpcid_single pti ssbd ibrs ibpb stibp tpr_shadow vnmi flexpriority ept vpid ept_ad fsgsbase tsc_adjust bmi1 avx2 smep bmi2 erms invpcid mpx rdseed adx smap clflushopt intel_pt xsaveopt xsavec xgetbv1 xsaves dtherm ida arat pln pts hwp hwp_notify hwp_act_window hwp_epp md_clear flush_l1d arch_capabilities
vmx flags	: vnmi preemption_timer invvpid ept_x_only ept_ad ept_1gb flexpriority tsc_offset vtpr mtf vapic ept vpid unrestricted_guest ple shadow_vmcs pml
bugs		: cpu_meltdown spectre_v1 spectre_v2 spec_store_bypass l1tf mds swapgs taa itlb_multihit srbds mmio_stale_data retbleed
bogomips	: 8003.30
clflush size	: 64
cache_alignment	: 64
address sizes	: 39 bits physical, 48 bits virtual
power management:

processor	: 3
vendor_id	: GenuineIntel
cpu family	: 6
model		: 94
model name	: Intel(R) Core(TM) i7-6700K CPU @ 4.00GHz
stepping	: 3
microcode	: 0xf0
cpu MHz		: 821.565
cache size	: 8192 KB
physical id	: 0
siblings	: 8
core id		: 3
cpu cores	: 4
apicid		: 6
initial apicid	: 6
fpu		: yes
fpu_exception	: yes
cpuid level	: 22
wp		: yes
flags		: fpu vme de pse tsc msr pae mce cx8 apic sep mtrr pge mca cmov pat pse36 clflush dts acpi mmx fxsr sse sse2 ss ht tm pbe syscall nx pdpe1gb rdtscp lm constant_tsc art arch_perfmon pebs bts rep_good nopl xtopology nonstop_tsc cpuid aperfmperf pni pclmulqdq dtes64 monitor ds_cpl vmx est tm2 ssse3 sdbg fma cx16 xtpr pdcm pcid sse4_1 sse4_2 x2apic movbe popcnt tsc_deadline_timer aes xsave avx f16c rdrand lahf_lm abm 3dnowprefetch cpuid_fault invpcid_single pti ssbd ibrs ibpb stibp tpr_shadow vnmi flexpriority ept vpid ept_ad fsgsbase tsc_adjust bmi1 avx2 smep bmi2 erms invpcid mpx rdseed adx smap clflushopt intel_pt xsaveopt xsavec xgetbv1 xsaves dtherm ida arat pln pts hwp hwp_notify hwp_act_window hwp_epp md_clear flush_l1d arch_capabilities
vmx flags	: vnmi preemption_timer invvpid ept_x_only ept_ad ept_1gb flexpriority tsc_offset vtpr mtf vapic ept vpid unrestricted_guest ple shadow_vmcs pml
bugs		: cpu_meltdown spectre_v1 spectre_v2 spec_store_bypass l1tf mds swapgs taa itlb_multihit srbds mmio_stale_data retbleed
bogomips	: 8003.30
clflush size	: 64
cache_alignment	: 64
address sizes	: 39 bits physical, 48 bits virtual
power management:

processor	: 4
vendor_id	: GenuineIntel
cpu family	: 6
model		: 94
model name	: Intel(R) Core(TM) i7-6700K CPU @ 4.00GHz
stepping	: 3
microcode	: 0xf0
cpu MHz		: 800.036
cache size	: 8192 KB
physical id	: 0
siblings	: 8
core id		: 0
cpu cores	: 4
apicid		: 1
initial apicid	: 1
fpu		: yes
fpu_exception	: yes
cpuid level	: 22
wp		: yes
flags		: fpu vme de pse tsc msr pae mce cx8 apic sep mtrr pge mca cmov pat pse36 clflush dts acpi mmx fxsr sse sse2 ss ht tm pbe syscall nx pdpe1gb rdtscp lm constant_tsc art arch_perfmon pebs bts rep_good nopl xtopology nonstop_tsc cpuid aperfmperf pni pclmulqdq dtes64 monitor ds_cpl vmx est tm2 ssse3 sdbg fma cx16 xtpr pdcm pcid sse4_1 sse4_2 x2apic movbe popcnt tsc_deadline_timer aes xsave avx f16c rdrand lahf_lm abm 3dnowprefetch cpuid_fault invpcid_single pti ssbd ibrs ibpb stibp tpr_shadow vnmi flexpriority ept vpid ept_ad fsgsbase tsc_adjust bmi1 avx2 smep bmi2 erms invpcid mpx rdseed adx smap clflushopt intel_pt xsaveopt xsavec xgetbv1 xsaves dtherm ida arat pln pts hwp hwp_notify hwp_act_window hwp_epp md_clear flush_l1d arch_capabilities
vmx flags	: vnmi preemption_timer invvpid ept_x_only ept_ad ept_1gb flexpriority tsc_offset vtpr mtf vapic ept vpid unrestricted_guest ple shadow_vmcs pml
bugs		: cpu_meltdown spectre_v1 spectre_v2 spec_store_bypass l1tf mds swapgs taa itlb_multihit srbds mmio_stale_data retbleed
bogomips	: 8003.30
clflush size	: 64
cache_alignment	: 64
address sizes	: 39 bits physical, 48 bits virtual
power management:

processor	: 5
vendor_id	: GenuineIntel
cpu family	: 6
model		: 94
model name	: Intel(R) Core(TM) i7-6700K CPU @ 4.00GHz
stepping	: 3
microcode	: 0xf0
cpu MHz		: 4000.000
cache size	: 8192 KB
physical id	: 0
siblings	: 8
core id		: 1
cpu cores	: 4
apicid		: 3
initial apicid	: 3
fpu		: yes
fpu_exception	: yes
cpuid level	: 22
wp		: yes
flags		: fpu vme de pse tsc msr pae mce cx8 apic sep mtrr pge mca cmov pat pse36 clflush dts acpi mmx fxsr sse sse2 ss ht tm pbe syscall nx pdpe1gb rdtscp lm constant_tsc art arch_perfmon pebs bts rep_good nopl xtopology nonstop_tsc cpuid aperfmperf pni pclmulqdq dtes64 monitor ds_cpl vmx est tm2 ssse3 sdbg fma cx16 xtpr pdcm pcid sse4_1 sse4_2 x2apic movbe popcnt tsc_deadline_timer aes xsave avx f16c rdrand lahf_lm abm 3dnowprefetch cpuid_fault invpcid_single pti ssbd ibrs ibpb stibp tpr_shadow vnmi flexpriority ept vpid ept_ad fsgsbase tsc_adjust bmi1 avx2 smep bmi2 erms invpcid mpx rdseed adx smap clflushopt intel_pt xsaveopt xsavec xgetbv1 xsaves dtherm ida arat pln pts hwp hwp_notify hwp_act_window hwp_epp md_clear flush_l1d arch_capabilities
vmx flags	: vnmi preemption_timer invvpid ept_x_only ept_ad ept_1gb flexpriority tsc_offset vtpr mtf vapic ept vpid unrestricted_guest ple shadow_vmcs pml
bugs		: cpu_meltdown spectre_v1 spectre_v2 spec_store_bypass l1tf mds swapgs taa itlb_multihit srbds mmio_stale_data retbleed
bogomips	: 8003.30
clflush size	: 64
cache_alignment	: 64
address sizes	: 39 bits physical, 48 bits virtual
power management:

processor	: 6
vendor_id	: GenuineIntel
cpu family	: 6
model		: 94
model name	: Intel(R) Core(TM) i7-6700K CPU @ 4.00GHz
stepping	: 3
microcode	: 0xf0
cpu MHz		: 800.019
cache size	: 8192 KB
physical id	: 0
siblings	: 8
core id		: 2
cpu cores	: 4
apicid		: 5
initial apicid	: 5
fpu		: yes
fpu_exception	: yes
cpuid level	: 22
wp		: yes
flags		: fpu vme de pse tsc msr pae mce cx8 apic sep mtrr pge mca cmov pat pse36 clflush dts acpi mmx fxsr sse sse2 ss ht tm pbe syscall nx pdpe1gb rdtscp lm constant_tsc art arch_perfmon pebs bts rep_good nopl xtopology nonstop_tsc cpuid aperfmperf pni pclmulqdq dtes64 monitor ds_cpl vmx est tm2 ssse3 sdbg fma cx16 xtpr pdcm pcid sse4_1 sse4_2 x2apic movbe popcnt tsc_deadline_timer aes xsave avx f16c rdrand lahf_lm abm 3dnowprefetch cpuid_fault invpcid_single pti ssbd ibrs ibpb stibp tpr_shadow vnmi flexpriority ept vpid ept_ad fsgsbase tsc_adjust bmi1 avx2 smep bmi2 erms invpcid mpx rdseed adx smap clflushopt intel_pt xsaveopt xsavec xgetbv1 xsaves dtherm ida arat pln pts hwp hwp_notify hwp_act_window hwp_epp md_clear flush_l1d arch_capabilities
vmx flags	: vnmi preemption_timer invvpid ept_x_only ept_ad ept_1gb flexpriority tsc_offset vtpr mtf vapic ept vpid unrestricted_guest ple shadow_vmcs pml
bugs		: cpu_meltdown spectre_v1 spectre_v2 spec_store_bypass l1tf mds swapgs taa itlb_multihit srbds mmio_stale_data retbleed
bogomips	: 8003.30
clflush size	: 64
cache_alignment	: 64
address sizes	: 39 bits physical, 48 bits virtual
power management:

processor	: 7
vendor_id	: GenuineIntel
cpu family	: 6
model		: 94
model name	: Intel(R) Core(TM) i7-6700K CPU @ 4.00GHz
stepping	: 3
microcode	: 0xf0
cpu MHz		: 4000.000
cache size	: 8192 KB
physical id	: 0
siblings	: 8
core id		: 3
cpu cores	: 4
apicid		: 7
initial apicid	: 7
fpu		: yes
fpu_exception	: yes
cpuid level	: 22
wp		: yes
flags		: fpu vme de pse tsc msr pae mce cx8 apic sep mtrr pge mca cmov pat pse36 clflush dts acpi mmx fxsr sse sse2 ss ht tm pbe syscall nx pdpe1gb rdtscp lm constant_tsc art arch_perfmon pebs bts rep_good nopl xtopology nonstop_tsc cpuid aperfmperf pni pclmulqdq dtes64 monitor ds_cpl vmx est tm2 ssse3 sdbg fma cx16 xtpr pdcm pcid sse4_1 sse4_2 x2apic movbe popcnt tsc_deadline_timer aes xsave avx f16c rdrand lahf_lm abm 3dnowprefetch cpuid_fault invpcid_single pti ssbd ibrs ibpb stibp tpr_shadow vnmi flexpriority ept vpid ept_ad fsgsbase tsc_adjust bmi1 avx2 smep bmi2 erms invpcid mpx rdseed adx smap clflushopt intel_pt xsaveopt xsavec xgetbv1 xsaves dtherm ida arat pln pts hwp hwp_notify hwp_act_window hwp_epp md_clear flush_l1d arch_capabilities
vmx flags	: vnmi preemption_timer invvpid ept_x_only ept_ad ept_1gb flexpriority tsc_offset vtpr mtf vapic ept vpid unrestricted_guest ple shadow_vmcs pml
bugs		: cpu_meltdown spectre_v1 spectre_v2 spec_store_bypass l1tf mds swapgs taa itlb_multihit srbds mmio_stale_data retbleed
bogomips	: 8003.30
clflush size	: 64
cache_alignment	: 64
address sizes	: 39 bits physical, 48 bits virtual
power management:

"
	);
        assert!(result.is_ok());
    }
}
