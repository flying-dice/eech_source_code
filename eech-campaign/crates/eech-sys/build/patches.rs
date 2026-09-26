//
// Source patches: the ONLY places where the native kernel executes something
// other than the original EECH text. Each patch replaces whole lines that must
// occur exactly `count` times in the original file; the build fails if the
// original text is not found as expected. The patched file keeps #line
// provenance to the original file and line numbers.
//
// Every patch is listed, with its evidence, in eech-campaign/docs/patches.md.
//

pub struct Patch {
    pub file: &'static str,
    pub id: &'static str,
    #[allow(dead_code)]
    pub why: &'static str,
    pub original: &'static str,
    pub replacement: &'static str,
    pub count: usize,
}

pub const PATCHES: &[Patch] = &[
    Patch {
        file: "aphavoc/source/entity/system/en_funcs/en_creat.c",
        id: "P1a-stack-attributes-storage",
        why: "64-bit blocker B1: the attribute list must be materialised into storage that lives as long as the original's argument stack did (this call frame)",
        original: "\tchar\n\t\t*pargs_buffer;\n",
        replacement: "\tchar\n\t\t*pargs_buffer;\n\n\tEECH_STACK_ATTRIBUTES_STORAGE (eech_stack_attributes);\n",
        count: 2,
    },
    Patch {
        file: "aphavoc/source/entity/system/en_funcs/en_creat.c",
        id: "P1b-stack-attributes-marshal",
        why: "64-bit blocker B1: `(char *) pargs` reinterprets a va_list as the i386 argument stack. On x86-64 SysV va_list is a register-save descriptor and on Win64 the slots are 8 bytes, so the attribute reader (get_list_item) reads garbage. The marshaller walks the same attribute grammar with va_arg and writes the buffer get_list_item expects.",
        original: "\t\tpargs_buffer = (char *) pargs;\n",
        replacement: "\t\tpargs_buffer = EECH_STACK_ATTRIBUTES (eech_stack_attributes, pargs);\n",
        count: 2,
    },
    Patch {
        file: "aphavoc/source/entity/special/keysite/ks_updt.c",
        id: "P2-keysite-task-timer",
        why: "global state G1: update_server's function-local `static float task_timer` is campaign state shared by every keysite and never saved or reset; it would leak from one campaign instance into the next. It becomes a named global that the kernel resets; the arithmetic is unchanged.",
        original: "\t\tstatic float task_timer = 0.0;\n",
        replacement: "\t\textern float eech_ks_updt_task_timer;\n#define task_timer eech_ks_updt_task_timer\n",
        count: 1,
    },
];

pub fn apply(file: &str, text: &str) -> String {
    let mut out = format!("#line 1 \"{file}\"\n");
    let mut rest = text;
    let mut line = 1usize;
    let patches: Vec<&Patch> = PATCHES.iter().filter(|p| p.file == file).collect();
    assert!(!patches.is_empty(), "no patch for {file}");
    for p in &patches {
        let found = text.matches(p.original).count();
        assert_eq!(
            found, p.count,
            "patch {} expects {} occurrence(s) of its original text in {}, found {}",
            p.id, p.count, file, found
        );
        assert!(p.original.ends_with('\n'), "patch {} must replace whole lines", p.id);
    }
    loop {
        // the earliest occurrence of any patch
        let next = patches
            .iter()
            .filter_map(|p| rest.find(p.original).map(|at| (at, *p)))
            .min_by_key(|(at, _)| *at);
        let Some((at, p)) = next else { break };
        assert!(at == 0 || rest.as_bytes()[at - 1] == b'\n', "patch {} must start at a line start", p.id);
        out.push_str(&rest[..at]);
        line += rest[..at].matches('\n').count();
        out.push_str(&format!("/* EECH-NATIVE PATCH {} (eech-campaign/docs/patches.md) */\n", p.id));
        out.push_str(p.replacement);
        line += p.original.matches('\n').count();
        out.push_str(&format!("#line {line} \"{file}\"\n"));
        rest = &rest[at + p.original.len()..];
    }
    out.push_str(rest);
    out
}
