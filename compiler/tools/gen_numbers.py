"""Writes the prelude's numeric classes. They differ only in name, range and which
conversions are implicit, so they are generated from one description.

A method is `@Intrinsic` only when it is one machine operation. Everything else is
written in the language:

- `Int` and `UInt64` have the machine's arithmetic, comparison, bit operations and
  conversions. Their other methods are written over those.
- The narrower integer classes have no arithmetic of their own. Each computes in `Int`
  and converts back, and the conversion raises when the result does not fit.
- The float classes have IEEE 754 arithmetic, rounding, and their conversions between
  machine numbers. Their conversions from `Rational`, their digits and their reading
  are written once in `Floats`.
- `Rational` is a numerator and a denominator of the prelude's `BigInt`.

Run from the repository root: python compiler/tools/gen_numbers.py
"""
import io

SIGNED = [("Int8", 8), ("Int16", 16), ("Int32", 32), ("Int", 64)]
UNSIGNED = [("UInt8", 8), ("UInt16", 16), ("UInt32", 32), ("UInt64", 64)]
FLOATS = ["Float32", "Float64"]
INTEGERS = [n for n, _ in SIGNED + UNSIGNED]
NARROW = [n for n, bits in SIGNED + UNSIGNED if bits < 64]
ALL = INTEGERS + FLOATS + ["Rational"]


def int_range(name):
    for n, bits in SIGNED:
        if n == name:
            return (-(1 << (bits - 1)), (1 << (bits - 1)) - 1)
    for n, bits in UNSIGNED:
        if n == name:
            return (0, (1 << bits) - 1)
    return None


def width(name):
    return dict(SIGNED + UNSIGNED)[name]


def implicit(src, dst):
    """Section 6.7: integer to a wider integer, Float32 to Float64, integer to Rational."""
    if src == dst:
        return False
    s, d = int_range(src), int_range(dst)
    if s and d:
        return d[0] <= s[0] and s[1] <= d[1]
    if s and dst == "Rational":
        return True
    return src == "Float32" and dst == "Float64"


# Conversions whose bodies are written out, by (class, method, source).
WRITTEN = {
    ("Int", "from", "Rational"): """return value.whole("Int").toInt();""",
    ("UInt64", "from", "Rational"): """return value.whole("UInt64").toUInt64();""",
    ("Float64", "from", "Rational"): """var near = nearest(value);
        if (near.isInfinite() || Rational.from(near) != value) {
            throw new ArithmeticException(value.toString() + " is not a value of Float64");
        }
        return near;""",
    ("Float64", "nearest", "Rational"): """return Floats.nearest(value, 53, -1074, 971);""",
    ("Float32", "from", "Rational"): """var wide = Float64.nearest(value);
        if (wide.isInfinite() || Rational.from(wide) != value || Float64.from(nearest(wide)) != wide) {
            throw new ArithmeticException(value.toString() + " is not a value of Float32");
        }
        return nearest(wide);""",
    # Rounded to a Float64 first, as the compiler rounds a constant.
    ("Float32", "nearest", "Rational"): """return nearest(Float64.nearest(value));""",
    ("Rational", "from", "Int"): """return new Rational(BigInt.from(value), BigInt.one());""",
    ("Rational", "from", "UInt64"): """return new Rational(BigInt.from(value), BigInt.one());""",
    ("Rational", "from", "Float64"): """if (value.isNaN() || value.isInfinite()) {
            throw new ArithmeticException(value.toString() + " is not a value of Rational");
        }
        if (value == 0) {
            return zero();
        }
        // The value is an integer times 2 to the power of the exponent of its lowest bit.
        var size = value.abs();
        var exponent = Floats.exponent(size, 53, -1074);
        var mantissa = BigInt.from(Int.from(Floats.scale(size, -exponent)));
        var signed = value < 0 ? -mantissa : mantissa;
        if (exponent >= 0) {
            return new Rational(signed.shiftLeft(exponent), BigInt.one());
        }
        return reduced(signed, BigInt.one().shiftLeft(-exponent));""",
}

COMMENTS = {
    ("Float64", "from", "Rational"): "The float equal to `value`, or ArithmeticException when there is none.",
    ("Float64", "nearest", "Rational"): "The float nearest to `value`. Of two as near, the one whose last bit is even;\n    // past the largest float, an infinity.",
    ("Float32", "nearest", "Rational"): "The Float32 nearest to the Float64 nearest to `value`, as the compiler rounds a\n    // constant.",
    ("Rational", "from", "Float64"): "The exact value of a float. NaN and the infinities raise.",
}


def conversion(name, src, method, machine):
    """One overload of `from` or `nearest`. `machine` lists the sources the class
    converts by itself. Every other source goes through one of them."""
    mark = "@Implicit\n    " if method == "from" and implicit(src, name) else ""
    head = f"public static {name} {method}({src} value)"
    if src == name:
        return f"    {mark}{head} {{\n        return value;\n    }}\n"
    if (name, method, src) in WRITTEN:
        comment = COMMENTS.get((name, method, src))
        comment = f"    // {comment}\n" if comment else ""
        return f"{comment}    {mark}{head} {{\n        {WRITTEN[(name, method, src)]}\n    }}\n"
    if src in machine:
        return f"    {mark}@Intrinsic\n    {head};\n"
    if src == "Float32":
        # Every Float32 is a Float64, so nothing is lost on the way.
        via = "Float64.from(value)"
    else:
        # Every integer but the largest UInt64 values is an Int. Those raise here,
        # and they fit no class that reaches this line.
        via = "Int.from(value)"
    return f"    {mark}{head} {{\n        return {method}({via});\n    }}\n"


def conversions(name, method, machine):
    return "\n".join(conversion(name, src, method, machine) for src in ALL)


HEADER = "// Generated by compiler/tools/gen_numbers.py. Edit the generator, not this file.\npackage cleat;\n\n"

# What every class that has `lessThan` gets from it.
ORDERED_BY_LESS_THAN = """
    @Override
    public Boolean atMost({n} other) {{
        return !(other < this);
    }}

    @Override
    public Boolean greaterThan({n} other) {{
        return other < this;
    }}

    @Override
    public Boolean atLeast({n} other) {{
        return !(this < other);
    }}

    @Override
    public Int compare({n} other) {{
        return this < other ? -1 : other < this ? 1 : 0;
    }}

    public {n} min({n} other) {{
        return this < other ? this : other;
    }}

    public {n} max({n} other) {{
        return this > other ? this : other;
    }}
"""

ZERO_AND_ONE = """
    @Override
    public static {n} zero() {{
        return 0;
    }}

    @Override
    public static {n} one() {{
        return 1;
    }}
"""

# Int and UInt64: the machine's operations, and the rest written over them.
WIDE = """    @Override
    @Intrinsic
    public {n} plus({n} other);

    @Override
    @Intrinsic
    public {n} minus({n} other);

    @Override
    @Intrinsic
    public {n} times({n} other);

    @Override
    public {n} negate() {{
        return zero() - this;
    }}
{zero_and_one}
    @Override
    @Intrinsic
    public Boolean lessThan({n} other);
{ordered}
    // The quotient rounded toward zero, and what is left of the dividend.
    @Intrinsic
    public {n} truncatingDiv({n} other);

    @Intrinsic
    public {n} truncatingRem({n} other);

    // The quotient rounded toward negative infinity: one less than the truncated
    // quotient when the division is inexact and the operands differ in sign.
    public {n} floorDiv({n} other) {{
        var quotient = truncatingDiv(other);
        var rest = truncatingRem(other);
        return rest != 0 && (rest < 0) != (other < 0) ? quotient - 1 : quotient;
    }}

    // Zero, or a value with the sign of the divisor.
    public {n} mod({n} other) {{
        var rest = truncatingRem(other);
        return rest != 0 && (rest < 0) != (other < 0) ? rest + other : rest;
    }}

    @Intrinsic
    public {n} wrappingPlus({n} other);

    @Intrinsic
    public {n} wrappingMinus({n} other);

    @Intrinsic
    public {n} wrappingTimes({n} other);

    @Intrinsic
    public {n} and({n} other);

    @Intrinsic
    public {n} or({n} other);

    @Intrinsic
    public {n} xor({n} other);

    public {n} complement() {{
        return this ^ {ones};
    }}

    @Intrinsic
    public {n} shiftLeft(Int count);

    @Intrinsic
    public {n} shiftRight(Int count);

    // The decimal digits, after a minus sign when the value is below zero. The digits
    // are taken from the value as it is, because the lowest Int has no negation.
    @Override
    public String toString() {{
        if (this == 0) {{
            return "0";
        }}
        var count = this < 0 ? 1 : 0;
        for (var rest = this; rest != 0; rest = rest.truncatingDiv(10)) {{
            count += 1;
        }}
        var chars = new Char[count];
        var left = this;
        for (var at = count - 1; left != 0; at--) {{
            var digit = Int.from(left.truncatingRem(10));
            chars[at] = Char.from(48 + (digit < 0 ? -digit : digit));
            left = left.truncatingDiv(10);
        }}
        if (this < 0) {{
            chars[0] = '-';
        }}
        return String.fromChars(chars, count);
    }}

    // The text of an integer in decimal, or null when the text is not one or is out of
    // range. A sign may come first.
    public static @Nullable {n} parse(String text) {{
        var negative = text.startsWith("-");
        var at = negative || text.startsWith("+") ? 1 : 0;
        if (at == text.length()) {{
            return null;
        }}
        {n} value = 0;
        try {{
            while (at < text.length()) {{
                var digit = text[at].code() - 48;
                if (digit < 0 || digit > 9) {{
                    return null;
                }}
                value = negative ? value * 10 - from(digit) : value * 10 + from(digit);
                at++;
            }}
        }} catch (ArithmeticException e) {{
            return null;
        }}
        return value;
    }}
"""

# The narrower integer classes: arithmetic in Int, and a conversion back.
NARROW_BODY = """    // The value of this class with the low {bits} bits of an Int. `from(Int)` is the
    // conversion that raises when the Int is out of range, and this one never raises.
    @Intrinsic
    private static {n} wrapping(Int value);

    @Override
    public {n} plus({n} other) {{
        return from(Int.from(this) + Int.from(other));
    }}

    @Override
    public {n} minus({n} other) {{
        return from(Int.from(this) - Int.from(other));
    }}

    @Override
    public {n} times({n} other) {{
        return from(Int.from(this) * Int.from(other));
    }}

    @Override
    public {n} negate() {{
        return from(-Int.from(this));
    }}
{zero_and_one}
    @Override
    public Boolean lessThan({n} other) {{
        return Int.from(this) < Int.from(other);
    }}
{ordered}
    public {n} floorDiv({n} other) {{
        return from(Int.from(this).floorDiv(Int.from(other)));
    }}

    public {n} mod({n} other) {{
        return from(Int.from(this).mod(Int.from(other)));
    }}

    public {n} truncatingDiv({n} other) {{
        return from(Int.from(this).truncatingDiv(Int.from(other)));
    }}

    public {n} truncatingRem({n} other) {{
        return from(Int.from(this).truncatingRem(Int.from(other)));
    }}

    public {n} wrappingPlus({n} other) {{
        return wrapping(Int.from(this) + Int.from(other));
    }}

    public {n} wrappingMinus({n} other) {{
        return wrapping(Int.from(this) - Int.from(other));
    }}

    public {n} wrappingTimes({n} other) {{
        return wrapping(Int.from(this) * Int.from(other));
    }}

    public {n} and({n} other) {{
        return wrapping(Int.from(this) & Int.from(other));
    }}

    public {n} or({n} other) {{
        return wrapping(Int.from(this) | Int.from(other));
    }}

    public {n} xor({n} other) {{
        return wrapping(Int.from(this) ^ Int.from(other));
    }}

    public {n} complement() {{
        return wrapping(~Int.from(this));
    }}

    // A count below zero, or not below the width of the class, raises.
    public {n} shiftLeft(Int count) {{
        if (count < 0 || count >= {bits}) {{
            throw new ArithmeticException("a shift count of " + count + " is not a value of a shift of this class");
        }}
        return wrapping(Int.from(this) << count);
    }}

    public {n} shiftRight(Int count) {{
        if (count < 0 || count >= {bits}) {{
            throw new ArithmeticException("a shift count of " + count + " is not a value of a shift of this class");
        }}
        return wrapping(Int.from(this) >> count);
    }}

    @Override
    public String toString() {{
        return Int.from(this).toString();
    }}

    // The text of an integer in decimal, or null when the text is not one or is out of
    // range. A sign may come first.
    public static @Nullable {n} parse(String text) {{
        var wide = Int.parse(text);
        if (wide == null || wide < {low} || wide > {high}) {{
            return null;
        }}
        return from(wide);
    }}
"""

FLOAT = """    @Override
    @Intrinsic
    public {n} plus({n} other);

    @Override
    @Intrinsic
    public {n} minus({n} other);

    @Override
    @Intrinsic
    public {n} times({n} other);

    @Override
    @Intrinsic
    public {n} div({n} other);

    // The value with the other sign. Subtracting from zero would lose the sign of zero.
    @Override
    @Intrinsic
    public {n} negate();
{zero_and_one}
    // False when either side is NaN.
    @Override
    @Intrinsic
    public Boolean lessThan({n} other);

    @Override
    @Intrinsic
    public Boolean atMost({n} other);

    @Override
    public Boolean greaterThan({n} other) {{
        return other < this;
    }}

    @Override
    public Boolean atLeast({n} other) {{
        return other <= this;
    }}

    // A total order in which NaN is greatest and the two zeros are equal.
    @Override
    public Int compare({n} other) {{
        if (this < other) {{
            return -1;
        }}
        if (other < this) {{
            return 1;
        }}
        if (isNaN()) {{
            return other.isNaN() ? 0 : 1;
        }}
        return other.isNaN() ? -1 : 0;
    }}

    // The order of IEEE 754 over every value, which tells the two zeros apart.
    @Intrinsic
    public Int totalOrder({n} other);

    @Intrinsic
    public {n} floor();

    @Intrinsic
    public {n} ceil();

    @Intrinsic
    public {n} truncate();

    @Intrinsic
    public {n} round();

    @Intrinsic
    public {n} sqrt();

    @Intrinsic
    public {n} abs();

    // NaN is the one value that is not at most itself.
    public Boolean isNaN() {{
        return !(this <= this);
    }}

    // An infinity less itself is NaN, and a finite value less itself is zero.
    public Boolean isInfinite() {{
        return !isNaN() && (this - this).isNaN();
    }}

    public {n} min({n} other) {{
        return this < other ? this : other;
    }}

    public {n} max({n} other) {{
        return this > other ? this : other;
    }}

    // The shortest digits that read back as the value, with a point and at least one
    // digit after it, or in exponent form from 1e21 up and below 1e-7. NaN and the
    // infinities are "NaN", "Infinity" and "-Infinity".
    @Override
    public String toString() {{
        var size = abs();
        return Floats.show({wide}, {precision}, {emin}, size > 0 && (size >= 1e21 || size < 1e-7));
    }}

    // The decimal digits of the value rounded to `places` digits after the point, ties to
    // even, after a minus sign when the value is below zero or is -0.0. NaN and the
    // infinities are written as toString writes them.
    public String toFixed(Int places) {{
        if (places < 0 || places > 1000) {{
            throw new IllegalArgumentException("" + places + " digits after the point");
        }}
        if (isNaN() || isInfinite()) {{
            return toString();
        }}
        var digits = Rational.from(abs()).toDecimal(places);
        return this < 0 || this == 0 && 1 / this < 0 ? "-" + digits : digits;
    }}

    // A decimal numeral, such as "-1.5", ".5" or "6.02e23", read as the nearest float,
    // or null when the text is not one.
    public static @Nullable {n} parse(String text) {{
        {parse}
    }}
"""

# What Floats is told about each float class: its precision, and the exponents of its
# least and greatest lowest bit.
FLOAT_SHAPE = {"Float64": (53, -1074, 971), "Float32": (24, -149, 104)}

RATIONAL = """    // The value in lowest terms: the denominator is above zero, and no integer above one
    // divides both. Each value has one form, so the defaults of a value class compare
    // values.
    package BigInt num;
    package BigInt den;

    private Rational {
    }

    // n/d in lowest terms. The denominator is not zero.
    private static Rational reduced(BigInt n, BigInt d) {
        var top = d.isNegative() ? -n : n;
        var bottom = d.isNegative() ? -d : d;
        var common = top.gcd(bottom);
        if (common.isOne()) {
            return new Rational(top, bottom);
        }
        return new Rational(top.floorDiv(common), bottom.floorDiv(common));
    }

    @Override
    public Rational plus(Rational other) {
        if (den.isOne() && other.den.isOne()) {
            return new Rational(num + other.num, den);
        }
        return reduced(num * other.den + other.num * den, den * other.den);
    }

    @Override
    public Rational minus(Rational other) {
        if (den.isOne() && other.den.isOne()) {
            return new Rational(num - other.num, den);
        }
        return reduced(num * other.den - other.num * den, den * other.den);
    }

    @Override
    public Rational times(Rational other) {
        if (den.isOne() && other.den.isOne()) {
            return new Rational(num * other.num, den);
        }
        return reduced(num * other.num, den * other.den);
    }

    // A zero divisor raises.
    @Override
    public Rational div(Rational other) {
        if (other.num.isZero()) {
            throw new ArithmeticException("division by zero");
        }
        return reduced(num * other.den, den * other.num);
    }

    @Override
    public Rational negate() {
        return new Rational(-num, den);
    }
{zero_and_one}
    @Override
    public Boolean lessThan(Rational other) {
        if (den == other.den) {
            return num < other.num;
        }
        return num * other.den < other.num * den;
    }
{ordered}
    // The value in lowest terms. The denominator is above zero.
    public Rational numerator() {
        return new Rational(num, BigInt.one());
    }

    public Rational denominator() {
        return new Rational(den, BigInt.one());
    }

    public Boolean isInteger() {
        return den.isOne();
    }

    // The integer this is. A value with a fraction raises as a conversion to the class
    // `target` does.
    package BigInt whole(String target) {
        if (!den.isOne()) {
            throw new ArithmeticException(toString() + " is not a value of " + target);
        }
        return num;
    }

    // The quotient rounded toward negative infinity. A zero divisor raises.
    public Rational floorDiv(Rational other) {
        if (other.num.isZero()) {
            throw new ArithmeticException("division by zero");
        }
        return new Rational((num * other.den).floorDiv(den * other.num), BigInt.one());
    }

    public Rational mod(Rational other) {
        return this - floorDiv(other) * other;
    }

    public Rational floor() {
        return new Rational(num.floorDiv(den), BigInt.one());
    }

    public Rational ceil() {
        return -(-this).floor();
    }

    public Rational truncate() {
        return this < zero() ? ceil() : floor();
    }

    // The nearest integer, ties to even.
    public Rational round() {
        return new Rational(nearestInteger(num, den), BigInt.one());
    }

    // The nearest multiple of ten to the power -places, ties to even.
    public Rational round(Int places) {
        Rational scale = 1;
        var count = places < 0 ? -places : places;
        for (var i = 0; i < count; i++) {
            scale = scale * 10;
        }
        return places < 0 ? (this / scale).round() * scale : (this * scale).round() / scale;
    }

    // The integer nearest to n/d, where d is above zero. Ties go to the even one.
    private static BigInt nearestInteger(BigInt n, BigInt d) {
        var below = n.floorDiv(d);
        var twice = n.mod(d).shiftLeft(1);
        if (twice > d || twice == d && below.mod(BigInt.from(2)).isOne()) {
            return below + BigInt.one();
        }
        return below;
    }

    public Rational abs() {
        return this < zero() ? -this : this;
    }

    // The digits of round(places), with exactly `places` digits after the point and a
    // minus sign when they are not all zero and the value is below zero.
    public String toDecimal(Int places) {
        if (places < 0 || places > 100000) {
            throw new IllegalArgumentException("" + places + " digits after the point");
        }
        var scaled = nearestInteger(num * BigInt.tenTo(places), den);
        // At least one digit before the point.
        var padded = new StringBuilder();
        for (var pad = scaled.abs().toString().length(); pad <= places; pad++) {
            padded.append('0');
        }
        padded.append(scaled.abs());
        var digits = padded.toString();
        var whole = digits.length() - places;
        var text = new StringBuilder();
        if (scaled.isNegative()) {
            text.append('-');
        }
        text.append(digits.substring(0, whole));
        if (places > 0) {
            text.append('.');
            text.append(digits.substring(whole, digits.length()));
        }
        return text.toString();
    }

    // The numerator, and "/" and the denominator unless it is one.
    @Override
    public String toString() {
        return den.isOne() ? num.toString() : num.toString() + "/" + den;
    }

    // A decimal numeral, such as "-12.50", or a fraction "a/b" of two integers, whose
    // digits `_` may separate. Whitespace may stand around the text and around each
    // integer of a fraction. Null when the text is neither, or the denominator is not
    // above zero.
    public static @Nullable Rational parse(String text) {
        var trimmed = text.trim();
        var slash = trimmed.indexOf('/');
        if (slash >= 0) {
            var top = BigInt.parse(trimmed.substring(0, slash).trim());
            var bottom = BigInt.parse(trimmed.substring(slash + 1, trimmed.length()).trim());
            if (top == null || bottom == null || bottom.sign() <= 0) {
                return null;
            }
            return reduced(top, bottom);
        }
        var negative = trimmed.startsWith("-");
        var body = negative || trimmed.startsWith("+") ? trimmed.substring(1, trimmed.length()) : trimmed;
        var point = body.indexOf('.');
        var whole = point < 0 ? body : body.substring(0, point);
        var fraction = point < 0 ? "" : body.substring(point + 1, body.length());
        if (whole.isEmpty() && fraction.isEmpty() || !allDigits(whole) || !allDigits(fraction)) {
            return null;
        }
        var digits = (BigInt) BigInt.parse(whole + fraction);
        return reduced(negative ? -digits : digits, BigInt.tenTo(fraction.length()));
    }

    private static Boolean allDigits(String text) {
        for (Char c : text) {
            if (!BigInt.isDigit(c)) {
                return false;
            }
        }
        return true;
    }

    // The value of a literal. The compiler writes it as "numerator/denominator" and
    // calls this once for each literal.
    package static Rational literal(String text) {
        var slash = text.indexOf('/');
        var top = (BigInt) BigInt.parse(text.substring(0, slash));
        var bottom = (BigInt) BigInt.parse(text.substring(slash + 1, text.length()));
        return reduced(top, bottom);
    }
"""


def write(name, body):
    io.open(f"prelude/{name}.cleat", "w", encoding="utf-8", newline="\n").write(HEADER + body)


def integer(name):
    ordered = ORDERED_BY_LESS_THAN.format(n=name)
    zero_and_one = ZERO_AND_ONE.format(n=name)
    low, high = int_range(name)
    head = f"public value class {name} implements Numeric<{name}>, Ordered<{name}> {{\n    private {name} {{\n    }}\n\n"
    if name in NARROW:
        body = NARROW_BODY.format(n=name, bits=width(name), low=low, high=high, zero_and_one=zero_and_one, ordered=ordered)
        machine = ["Int"]
    else:
        # The complement of an Int is its bits with -1; of a UInt64, with every bit set.
        ones = "-1" if low < 0 else str(high)
        body = WIDE.format(n=name, ones=ones, zero_and_one=zero_and_one, ordered=ordered)
        # Int takes every narrower integer as it is, and checks the rest. UInt64 checks
        # an Int, and has its own way from the classes an Int cannot hold.
        machine = NARROW + ["UInt64", "Float64"] if name == "Int" else ["Int", "Float64"]
    write(name, head + body + "\n" + conversions(name, "from", machine) + "}\n")


for name in INTEGERS:
    integer(name)

for name in FLOATS:
    other = "Float64" if name == "Float32" else "Float32"
    # An Int and a UInt64 are converted in one step, because going through the other
    # float class would round twice.
    machine = ["Int", "UInt64", other]
    write(
        name,
        f"public value class {name} implements Divisible<{name}>, Ordered<{name}> {{\n"
        f"    private {name} {{\n    }}\n\n"
        + FLOAT.format(
            n=name,
            zero_and_one=ZERO_AND_ONE.format(n=name),
            wide="this" if name == "Float64" else "Float64.from(this)",
            precision=FLOAT_SHAPE[name][0],
            emin=FLOAT_SHAPE[name][1],
            parse="return Floats.parse(text, 53, -1074, 971);"
            if name == "Float64"
            else "var wide = Floats.parse(text, 24, -149, 104);\n        return wide == null ? null : from(wide);",
        )
        + "\n"
        + conversions(name, "from", machine)
        + "\n"
        + conversions(name, "nearest", machine)
        + "}\n",
    )

write(
    "Rational",
    "public value class Rational implements Divisible<Rational>, Ordered<Rational> {\n"
    + RATIONAL.replace("{zero_and_one}", ZERO_AND_ONE.format(n="Rational")).replace("{ordered}", ORDERED_BY_LESS_THAN.format(n="Rational"))
    + "\n"
    + conversions("Rational", "from", [])
    + "}\n",
)
