import re

# using sc_AdminId = sc_TypeId<sc_Admin>;
HEADER = "squire_core.h"
POLY_MORPHISM_RE = re.compile(
    "using (sc_[a-zA-Z]+Id) = sc_TypeId<(sc_[a-zA-Z]+)>;", re.I
)


def main() -> None:
    # Read
    f = open(HEADER, "r")
    data = f.read()
    f.close()

    # Process the header
    ret = ""
    for line in data.split("\n"):
        mtchs = POLY_MORPHISM_RE.match(line)
        if mtchs:
            (
                type,
                _,
            ) = mtchs.groups()
            ret += f"using {type} = struct __{type}" + "{ sc_Uuid _0; };"
        else:
            ret += line
        ret += "\n"

    # Write
    f = open(HEADER, "w")
    f.write(ret)
    f.close()


if __name__ == "__main__":
    main()
