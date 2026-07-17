use super::*;

pub static ABYSS: LazyLock<Schema> = LazyLock::new(abyss);

pub static BLACK: LazyLock<Schema> = LazyLock::new(black);

pub static EMBER: LazyLock<Schema> = LazyLock::new(ember);

pub static LUXURY: LazyLock<Schema> = LazyLock::new(luxury);

pub static FANTASY: LazyLock<Schema> = LazyLock::new(fantasy);

pub static WINTER: LazyLock<Schema> = LazyLock::new(winter);

pub static AUTUMN: LazyLock<Schema> = LazyLock::new(autumn);

fn abyss() -> Schema {
    Schema {
        background: Triplet {
            weak: Pair {
                color: color!(0, 30, 41),
                text: color!(255, 214, 167),
            },
            base: Pair {
                color: color!(0, 17, 29),
                text: color!(255, 214, 167),
            },
            strong: Pair {
                color: color!(0, 6, 17),
                text: color!(255, 214, 167),
            },
        },
        primary: Triplet::generate(Pair {
            color: color!(189, 255, 0),
            text: color!(66, 118, 0),
        }),
        secondary: Triplet::generate(Pair {
            color: color!(206, 190, 244),
            text: color!(86, 71, 117),
        }),
        accent: Triplet::generate(Pair {
            color: color!(80, 80, 80),
            text: color!(248, 248, 248),
        }),
        neutral: Triplet::generate(Pair {
            color: color!(0, 56, 67),
            text: color!(255, 214, 167),
        }),
        info: Triplet::generate(Pair {
            color: color!(0, 186, 254),
            text: color!(4, 46, 73),
        }),
        success: Triplet::generate(Pair {
            color: color!(1, 223, 114),
            text: color!(2, 45, 20),
        }),
        warning: Triplet::generate(Pair {
            color: color!(255, 191, 0),
            text: color!(133, 66, 0),
        }),
        danger: Triplet::generate(Pair {
            color: color!(240, 78, 79),
            text: color!(105, 0, 0),
        }),
        radii: Radii {
            boxes: 8.0,
            fields: 8.0,
            selectors: 4.0,
        },
        is_dark: true,
    }
}

fn black() -> Schema {
    Schema {
        background: Triplet {
            weak: Pair {
                color: color!(20, 20, 20),
                text: color!(214, 214, 214),
            },
            base: Pair {
                color: color!(0, 0, 0),
                text: color!(214, 214, 214),
            },
            strong: Pair {
                color: color!(0, 0, 0),
                text: color!(214, 214, 214),
            },
        },
        primary: Triplet::generate(Pair {
            color: color!(58, 58, 58),
            text: color!(255, 255, 255),
        }),
        secondary: Triplet::generate(Pair {
            color: color!(58, 58, 58),
            text: color!(255, 255, 255),
        }),
        accent: Triplet::generate(Pair {
            color: color!(58, 58, 58),
            text: color!(255, 255, 255),
        }),
        neutral: Triplet::generate(Pair {
            color: color!(58, 58, 58),
            text: color!(255, 255, 255),
        }),
        info: Triplet::generate(Pair {
            color: color!(0, 0, 255),
            text: color!(198, 219, 255),
        }),
        success: Triplet::generate(Pair {
            color: color!(2, 128, 2),
            text: color!(211, 230, 208),
        }),
        warning: Triplet::generate(Pair {
            color: color!(255, 255, 0),
            text: color!(22, 22, 0),
        }),
        danger: Triplet::generate(Pair {
            color: color!(255, 3, 1),
            text: color!(22, 0, 0),
        }),
        radii: Radii {
            boxes: 4.0,
            fields: 4.0,
            selectors: 4.0,
        },
        is_dark: true,
    }
}

fn ember() -> Schema {
    Schema {
        background: Triplet {
            weak: Pair {
                color: color!(27, 27, 27),
                text: color!(214, 214, 214),
            },
            base: Pair {
                color: color!(0, 0, 0),
                text: color!(214, 214, 214),
            },
            strong: Pair {
                color: color!(11, 11, 11),
                text: color!(214, 214, 214),
            },
        },
        primary: Triplet::generate(Pair {
            color: color!(255, 103, 0),
            text: color!(19, 22, 22),
        }),
        secondary: Triplet::generate(Pair {
            color: color!(60, 90, 120),
            text: color!(255, 255, 255),
        }),
        accent: Triplet::generate(Pair {
            color: color!(253, 207, 43),
            text: color!(0, 0, 0),
        }),
        neutral: Triplet::generate(Pair {
            color: color!(58, 58, 58),
            text: color!(255, 255, 255),
        }),
        info: Triplet::generate(Pair {
            color: color!(25, 58, 183),
            text: color!(198, 219, 255),
        }),
        success: Triplet::generate(Pair {
            color: color!(2, 128, 2),
            text: color!(211, 230, 208),
        }),
        warning: Triplet::generate(Pair {
            color: color!(255, 167, 0),
            text: color!(22, 22, 0),
        }),
        danger: Triplet::generate(Pair {
            color: color!(191, 0, 4),
            text: color!(22, 0, 0),
        }),
        radii: Radii {
            boxes: 4.0,
            fields: 4.0,
            selectors: 4.0,
        },
        is_dark: true,
    }
}

fn luxury() -> Schema {
    Schema {
        background: Triplet {
            weak: Pair {
                color: color!(30, 29, 31),
                text: color!(220, 165, 77),
            },
            base: Pair {
                color: color!(9, 9, 11),
                text: color!(220, 165, 77),
            },
            strong: Pair {
                color: color!(23, 22, 24),
                text: color!(220, 165, 77),
            },
        },
        primary: Triplet::generate(Pair {
            color: color!(255, 255, 255),
            text: color!(22, 22, 22),
        }),
        secondary: Triplet::generate(Pair {
            color: color!(21, 39, 71),
            text: color!(203, 208, 215),
        }),
        accent: Triplet::generate(Pair {
            color: color!(81, 52, 72),
            text: color!(218, 211, 215),
        }),
        neutral: Triplet::generate(Pair {
            color: color!(51, 24, 0),
            text: color!(255, 231, 164),
        }),
        info: Triplet::generate(Pair {
            color: color!(103, 198, 255),
            text: color!(4, 14, 22),
        }),
        success: Triplet::generate(Pair {
            color: color!(135, 208, 58),
            text: color!(6, 16, 1),
        }),
        warning: Triplet::generate(Pair {
            color: color!(226, 213, 99),
            text: color!(18, 16, 3),
        }),
        danger: Triplet::generate(Pair {
            color: color!(255, 111, 111),
            text: color!(22, 4, 4),
        }),
        radii: Radii {
            boxes: 16.0,
            fields: 8.0,
            selectors: 16.0,
        },
        is_dark: true,
    }
}

fn fantasy() -> Schema {
    Schema {
        background: Triplet {
            weak: Pair {
                color: color!(255, 255, 255),
                text: color!(31, 41, 55),
            },
            base: Pair {
                color: color!(232, 232, 232),
                text: color!(31, 41, 55),
            },
            strong: Pair {
                color: color!(209, 209, 209),
                text: color!(31, 41, 55),
            },
        },
        primary: Triplet::generate(Pair {
            color: color!(109, 0, 118),
            text: color!(227, 206, 228),
        }),
        secondary: Triplet::generate(Pair {
            color: color!(0, 117, 194),
            text: color!(207, 228, 244),
        }),
        accent: Triplet::generate(Pair {
            color: color!(255, 134, 0),
            text: color!(24, 6, 0),
        }),
        neutral: Triplet::generate(Pair {
            color: color!(31, 41, 55),
            text: color!(205, 208, 211),
        }),
        info: Triplet::generate(Pair {
            color: color!(0, 181, 255),
            text: color!(0, 0, 0),
        }),
        success: Triplet::generate(Pair {
            color: color!(0, 169, 110),
            text: color!(0, 0, 0),
        }),
        warning: Triplet::generate(Pair {
            color: color!(255, 190, 0),
            text: color!(0, 0, 0),
        }),
        danger: Triplet::generate(Pair {
            color: color!(255, 88, 97),
            text: color!(0, 0, 0),
        }),
        radii: Radii {
            boxes: 16.0,
            fields: 8.0,
            selectors: 16.0,
        },
        is_dark: false,
    }
}

fn winter() -> Schema {
    Schema {
        background: Triplet {
            weak: Pair {
                color: color!(255, 255, 255),
                text: color!(57, 78, 106),
            },
            base: Pair {
                color: color!(242, 247, 254),
                text: color!(57, 78, 106),
            },
            strong: Pair {
                color: color!(227, 233, 244),
                text: color!(57, 78, 106),
            },
        },
        primary: Triplet::generate(Pair {
            color: color!(0, 105, 255),
            text: color!(206, 228, 255),
        }),
        secondary: Triplet::generate(Pair {
            color: color!(70, 58, 162),
            text: color!(213, 215, 238),
        }),
        accent: Triplet::generate(Pair {
            color: color!(193, 72, 172),
            text: color!(14, 2, 11),
        }),
        neutral: Triplet::generate(Pair {
            color: color!(2, 20, 49),
            text: color!(197, 203, 210),
        }),
        info: Triplet::generate(Pair {
            color: color!(148, 231, 251),
            text: color!(8, 19, 21),
        }),
        success: Triplet::generate(Pair {
            color: color!(120, 207, 209),
            text: color!(6, 15, 16),
        }),
        warning: Triplet::generate(Pair {
            color: color!(239, 215, 188),
            text: color!(20, 17, 13),
        }),
        danger: Triplet::generate(Pair {
            color: color!(229, 139, 139),
            text: color!(18, 7, 7),
        }),
        radii: Radii {
            boxes: 16.0,
            fields: 8.0,
            selectors: 16.0,
        },
        is_dark: false,
    }
}

fn autumn() -> Schema {
    Schema {
        background: Triplet {
            weak: Pair {
                color: color!(241, 241, 241),
                text: color!(20, 20, 20),
            },
            base: Pair {
                color: color!(219, 219, 219),
                text: color!(20, 20, 20),
            },
            strong: Pair {
                color: color!(197, 197, 197),
                text: color!(20, 20, 20),
            },
        },
        primary: Triplet::generate(Pair {
            color: color!(140, 3, 39),
            text: color!(237, 208, 208),
        }),
        secondary: Triplet::generate(Pair {
            color: color!(216, 82, 81),
            text: color!(17, 2, 2),
        }),
        accent: Triplet::generate(Pair {
            color: color!(213, 155, 107),
            text: color!(16, 9, 4),
        }),
        neutral: Triplet::generate(Pair {
            color: color!(130, 106, 92),
            text: color!(229, 224, 221),
        }),
        info: Triplet::generate(Pair {
            color: color!(68, 173, 187),
            text: color!(2, 11, 13),
        }),
        success: Triplet::generate(Pair {
            color: color!(73, 147, 128),
            text: color!(2, 8, 6),
        }),
        warning: Triplet::generate(Pair {
            color: color!(233, 127, 22),
            text: color!(19, 6, 0),
        }),
        danger: Triplet::generate(Pair {
            color: color!(255, 212, 209),
            text: color!(212, 0, 20),
        }),
        radii: Radii {
            boxes: 16.0,
            fields: 8.0,
            selectors: 16.0,
        },
        is_dark: false,
    }
}
