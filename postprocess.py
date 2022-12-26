import re

# using sc_AdminId = sc_TypeId<sc_Admin>;
HEADER = "squire_core.h"
POLY_MORPHISM_RE = re.compile(
    r"\s*using (sc_[a-zA-Z]+Id) = sc_TypeId<(sc_[a-zA-Z]+)>;\s*", re.I
)
RATIONAL_LINE_RE = re.compile(r"\s*using sc_r64 = sc_Rational32;\s*", re.I)


def main() -> None:
    # Read
    f = open(HEADER, "r")
    data = f.read()
    f.close()

    # Process the header
    ret = ""
    for line in data.split("\n"):
        mtchs = RATIONAL_LINE_RE.match(line)
        if mtchs:
            ret += """
/// This is ghastly wrapper for ratio32
typedef struct ratio32 {
    int _0;
    int _1;
} ratio32;

/// Turns a rational32 or ratio32 into a floating point number
float ratio32ToFloat(ratio32 r);

using sc_Rational32 = ratio32;\n""" + line + "\n"
            print("Rational32 is no longer undefined")
            continue

        mtchs = POLY_MORPHISM_RE.match(line)
        if mtchs:
            (
                type,
                _,
            ) = mtchs.groups()
            ret += f"using {type} = struct __{type}" + "{ sc_Uuid _0; };"
            print(f"{type} is no longer polymorphic")
        else:
            ret += line
        ret += "\n"

    # Write
    f = open(HEADER, "w")
    f.write(ret)
    f.close()


if __name__ == "__main__":
    print(f"Removing polymorphism from {HEADER}")
    main()
    print(f"{HEADER} has had all polymorphism removed.")
