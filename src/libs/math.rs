pub fn abs(x: f64) -> f64 {
    return f64::from_bits(x.to_bits() & (u64::MAX / 2));
}

const TOINT: f64 = 1. / f64::EPSILON;

pub fn floor(x: f64) -> f64 {
    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        not(target_feature = "sse2")
    ))]
    {
        if abs(x).to_bits() < 4503599627370496.0_f64.to_bits() {
            let truncated = x as i64 as f64;
            if truncated > x {
                return truncated - 1.0;
            } else {
                return truncated;
            }
        } else {
            return x;
        }
    }

    let ui = x.to_bits();
    let e = ((ui >> 52) & 0x7FF) as i32;

    if (e >= 0x3FF + 52) || (x == 0.) {
        return x;
    }

    let y = if (ui >> 63) != 0 {
        x - TOINT + TOINT - x
    } else {
        x + TOINT + TOINT - x
    };

    if e < 0x3FF {
        return if (ui >> 63) != 0 { -1. } else { 0. };
    }

    if y > 0. {
        return x + y - 1.;
    } else {
        return x + y;
    }
}

pub fn ceil(x: f64) -> f64 {
    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        not(target_feature = "sse2")
    ))]
    {
        if abs(x).to_bits() < 4503599627370496.0_f64.to_bits() {
            let truncated = x as i64 as f64;
            if truncated < x {
                return truncated + 1.0;
            } else {
                return truncated;
            }
        } else {
            return x;
        }
    }

    let u: u64 = x.to_bits();
    let e: i64 = (u >> 52 & 0x7ff) as i64;
    let y: f64;

    if e >= 0x3ff + 52 || x == 0. {
        return x;
    }

    y = if (u >> 63) != 0 {
        x - TOINT + TOINT - x
    } else {
        x + TOINT - TOINT - x
    };

    if e < 0x3ff {
        return if (u >> 63) != 0 { -0. } else { 1. };
    }

    if y < 0. {
        return x + y + 1.;
    } else {
        return x + y;
    }
}
