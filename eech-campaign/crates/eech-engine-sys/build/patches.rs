//
// Source patches of the headless engine build: the ONLY places where the
// engine compiles something other than the original EECH text. Each patch
// replaces text that must occur exactly `count` times in the original file;
// the build fails if it does not. Every patch is listed, with its evidence,
// in eech-campaign/docs/engine.md.
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
        file: "modules/system/debug.h",
        id: "E1-debug-log-macros",
        why: "T1 (docs/64-bit.md): the WIN32 release debug_log macros are `#define debug_log();`, called with arguments. Only MSVC's preprocessor accepts that. GCC-family compilers take EECH's own variadic definitions.",
        original: "#ifdef WIN32\n# ifdef __BORLANDC__\n",
        replacement: "/* EECH headless (E1): GCC-family compilers take the variadic definitions below */\n#if defined(WIN32) && !defined(__GNUC__)\n# ifdef __BORLANDC__\n",
        count: 1,
    },
    Patch {
        file: "aphavoc/source/ai/highlevl/highlevl.h",
        id: "E2-ai-log-macro",
        why: "T1 (docs/64-bit.md): the WIN32 release ai_log is `#define ai_log();`, called with arguments. GCC-family compilers take EECH's own variadic definition.",
        original: "#ifdef WIN32\n#define ai_log();\n",
        replacement: "/* EECH headless (E2): GCC-family compilers take the variadic definition below */\n#if defined(WIN32) && !defined(__GNUC__)\n#define ai_log();\n",
        count: 1,
    },
    Patch {
        file: "aphavoc/source/entity/special/effect/explosn/xp_dbase.h",
        id: "E3-explosion-union-member-names",
        why: "Two anonymous structs of one union both declare `frequency` and `smoke_lifetime`. MSVC's C++ front end accepts the repeated names (both at offsets 4 and 8); C rejects them. The second struct's copies are renamed; the layout and every access (which resolves to the first struct's members at the same offsets) are unchanged.",
        original: "\t\t\t\t\t\tgenerator_lifetime,\n\t\t\t\t\t\tfrequency,\n\t\t\t\t\t\tsmoke_lifetime;\n",
        replacement: "\t\t\t\t\t\tgenerator_lifetime,\n\t\t\t\t\t\t/* EECH headless (E3): same offsets as the first struct's frequency and smoke_lifetime */\n\t\t\t\t\t\tfrequency_shared_with_first_layout,\n\t\t\t\t\t\tsmoke_lifetime_shared_with_first_layout;\n",
        count: 1,
    },
    Patch {
        file: "aphavoc/source/entity/system/en_funcs/en_creat.c",
        id: "P1a-stack-attributes-include",
        why: "64-bit blocker B1 (docs/64-bit.md, docs/patches.md P1): declares the attribute-list marshaller.",
        original: "#include \"project.h\"\n",
        replacement: "#include \"project.h\"\n\n/* EECH headless (P1) */\n#include \"eech_attrs.h\"\n",
        count: 1,
    },
    Patch {
        file: "aphavoc/source/entity/system/en_funcs/en_creat.c",
        id: "P1b-stack-attributes-storage",
        why: "64-bit blocker B1: storage for the marshalled attribute list, living as long as the original's argument stack did (this call frame).",
        original: "\tchar\n\t\t*pargs_buffer;\n",
        replacement: "\tchar\n\t\t*pargs_buffer;\n\n\t/* EECH headless (P1) */\n\tchar\n\t\teech_stack_attributes[EECH_STACK_ATTRIBUTES_SIZE];\n",
        count: 2,
    },
    Patch {
        file: "aphavoc/source/entity/system/en_funcs/en_creat.c",
        id: "P1c-stack-attributes-marshal",
        why: "64-bit blocker B1: `(char *) pargs` reinterprets a va_list as the i386 argument stack; on x86-64 it is a descriptor. The marshaller (csrc/eech_attrs.c) walks the attribute grammar with va_arg and writes the list get_list_item expects.",
        original: "\t\tpargs_buffer = (char *) pargs;\n",
        replacement: "\t\t/* EECH headless (P1) */\n\t\tpargs_buffer = eech_marshal_stack_attributes (eech_stack_attributes, sizeof (eech_stack_attributes), pargs);\n",
        count: 2,
    },
];

pub fn apply(file: &str, text: &str) -> String {
    let mut out = text.to_string();
    for p in PATCHES.iter().filter(|p| p.file == file) {
        let found = out.matches(p.original).count();
        assert_eq!(found, p.count, "patch {} expects {} occurrence(s) of its original text in {}, found {}", p.id, p.count, file, found);
        out = out.replace(p.original, p.replacement);
    }
    out
}

pub fn patched_files() -> Vec<&'static str> {
    let mut files: Vec<&str> = PATCHES.iter().map(|p| p.file).collect();
    files.dedup();
    files
}
