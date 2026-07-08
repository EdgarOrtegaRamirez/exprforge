//! Unit system with dimensional analysis and conversion.
//!
//! Each unit is defined by its dimension (a vector of exponents for each base
//! quantity) and a conversion factor to the SI base unit. Temperature units
//! also have an offset. This allows for:
//!
//! - Converting between any two units of the same dimension
//! - Performing dimensional analysis on expressions
//! - Detecting unit errors (e.g., adding meters to kilograms)

use std::collections::HashMap;
use std::fmt;

/// The base physical quantities used for dimensional analysis.
///
/// Order matters: [Length, Mass, Time, Temperature, Current, Amount, Angle, Data]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BaseQuantity {
    Length,
    Mass,
    Time,
    Temperature,
    Current,
    Amount,
    Angle,
    Data,
}

/// A dimension represents the physical dimension of a quantity.
///
/// Each component is the exponent of the corresponding base quantity.
/// For example, speed (m/s) has dimension [1, 0, -1, 0, 0, 0, 0, 0].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dimension {
    /// Exponents for [Length, Mass, Time, Temperature, Current, Amount, Angle, Data]
    pub exponents: [i32; 8],
}

impl Dimension {
    /// Create a dimensionless quantity (all exponents zero).
    pub fn dimensionless() -> Self {
        Dimension { exponents: [0; 8] }
    }

    /// Create a dimension with a single base quantity.
    pub fn base(qty: BaseQuantity, exp: i32) -> Self {
        let mut d = Self::dimensionless();
        d.exponents[qty as usize] = exp;
        d
    }

    /// Check if this is a dimensionless quantity.
    pub fn is_dimensionless(&self) -> bool {
        self.exponents.iter().all(|&e| e == 0)
    }

    /// Multiply two dimensions (add exponents).
    pub fn multiply(&self, other: &Self) -> Self {
        let mut result = Self::dimensionless();
        for i in 0..8 {
            result.exponents[i] = self.exponents[i] + other.exponents[i];
        }
        result
    }

    /// Divide two dimensions (subtract exponents).
    pub fn divide(&self, other: &Self) -> Self {
        let mut result = Self::dimensionless();
        for i in 0..8 {
            result.exponents[i] = self.exponents[i] - other.exponents[i];
        }
        result
    }

    /// Raise a dimension to a power (multiply exponents).
    pub fn power(&self, n: i32) -> Self {
        let mut result = Self::dimensionless();
        for i in 0..8 {
            result.exponents[i] = self.exponents[i] * n;
        }
        result
    }

    /// Check if two dimensions are compatible (the same).
    pub fn is_compatible(&self, other: &Self) -> bool {
        self.exponents == other.exponents
    }
}

impl fmt::Display for Dimension {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_dimensionless() {
            return write!(f, "");
        }

        let names = ["m", "kg", "s", "K", "A", "mol", "rad", "bit"];
        let mut first = true;

        // Positive exponents first
        for (i, name) in names.iter().enumerate() {
            if self.exponents[i] > 0 {
                if !first {
                    write!(f, "·")?;
                }
                first = false;
                if self.exponents[i] == 1 {
                    write!(f, "{}", name)?;
                } else {
                    write!(f, "{}^{}", name, self.exponents[i])?;
                }
            }
        }

        // Negative exponents
        let mut has_neg = false;
        for (i, name) in names.iter().enumerate() {
            if self.exponents[i] < 0 {
                if !has_neg {
                    write!(f, "/")?;
                    has_neg = true;
                } else {
                    write!(f, "·")?;
                }
                if self.exponents[i] == -1 {
                    write!(f, "{}", name)?;
                } else {
                    write!(f, "{}^{}", name, -self.exponents[i])?;
                }
            }
        }

        if first && !has_neg {
            write!(f, "")?;
        }

        Ok(())
    }
}

/// A unit definition with its conversion factor and dimension.
#[derive(Debug, Clone)]
pub struct UnitDef {
    /// The symbol or name of the unit (e.g., "m", "ft", "kg").
    pub symbol: String,
    /// Full name of the unit (e.g., "meter", "foot").
    pub name: String,
    /// The dimension of this unit.
    pub dimension: Dimension,
    /// Conversion factor to the SI base unit.
    /// To convert from this unit to SI: value_si = value * factor + offset
    pub factor: f64,
    /// Offset for temperature units (0 for non-temperature).
    pub offset: f64,
    /// Category for grouping (e.g., "length", "mass").
    pub category: String,
}

/// The unit registry containing all known units.
pub struct UnitRegistry {
    units: HashMap<String, UnitDef>,
    /// Aliases: maps alternative names to canonical symbols.
    aliases: HashMap<String, String>,
}

impl UnitRegistry {
    /// Create a new unit registry with all built-in units.
    pub fn new() -> Self {
        let mut registry = UnitRegistry {
            units: HashMap::new(),
            aliases: HashMap::new(),
        };
        registry.register_all();
        registry
    }

    /// Look up a unit by its symbol or alias.
    pub fn get(&self, name: &str) -> Option<&UnitDef> {
        let canonical = self.aliases.get(name).map(|s| s.as_str()).unwrap_or(name);
        self.units.get(canonical)
    }

    /// Get all units in a given category.
    pub fn by_category(&self, category: &str) -> Vec<&UnitDef> {
        self.units
            .values()
            .filter(|u| u.category == category)
            .collect()
    }

    /// Get all categories.
    pub fn categories(&self) -> Vec<String> {
        let mut cats: Vec<String> = self
            .units
            .values()
            .map(|u| u.category.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        cats.sort();
        cats
    }

    /// Get all unit symbols.
    pub fn all_symbols(&self) -> Vec<String> {
        let mut syms: Vec<String> = self.units.keys().cloned().collect();
        syms.sort();
        syms
    }

    /// Convert a value from one unit to another.
    ///
    /// Returns an error if the units are not compatible (different dimensions).
    pub fn convert(&self, value: f64, from: &str, to: &str) -> Result<f64, String> {
        let from_unit = self
            .get(from)
            .ok_or_else(|| format!("Unknown unit: {}", from))?;
        let to_unit = self
            .get(to)
            .ok_or_else(|| format!("Unknown unit: {}", to))?;

        if !from_unit.dimension.is_compatible(&to_unit.dimension) {
            return Err(format!(
                "Cannot convert {} to {} (incompatible dimensions: {} vs {})",
                from, to, from_unit.dimension, to_unit.dimension
            ));
        }

        // Convert to SI base: value_si = value * factor + offset
        let value_si = value * from_unit.factor + from_unit.offset;

        // Convert from SI base to target: value = (value_si - offset) / factor
        let result = (value_si - to_unit.offset) / to_unit.factor;

        Ok(result)
    }

    /// Get the SI base value and dimension for a value in a given unit.
    pub fn to_base(&self, value: f64, unit: &str) -> Result<(f64, Dimension), String> {
        let unit_def = self
            .get(unit)
            .ok_or_else(|| format!("Unknown unit: {}", unit))?;
        Ok((
            value * unit_def.factor + unit_def.offset,
            unit_def.dimension.clone(),
        ))
    }

    /// Convert a value from SI base to a target unit.
    pub fn from_base(&self, value: f64, dimension: &Dimension, to: &str) -> Result<f64, String> {
        let to_unit = self
            .get(to)
            .ok_or_else(|| format!("Unknown unit: {}", to))?;

        if !to_unit.dimension.is_compatible(dimension) {
            return Err(format!(
                "Cannot convert to {} (incompatible dimensions: {} vs {})",
                to, dimension, to_unit.dimension
            ));
        }

        Ok((value - to_unit.offset) / to_unit.factor)
    }

    fn register(&mut self, def: UnitDef) {
        let symbol = def.symbol.clone();
        self.aliases.insert(symbol.clone(), symbol.clone());
        self.units.insert(symbol, def);
    }

    fn register_with_aliases(&mut self, def: UnitDef, aliases: &[&str]) {
        let symbol = def.symbol.clone();
        for alias in aliases {
            self.aliases.insert(alias.to_string(), symbol.clone());
        }
        self.aliases.insert(symbol.clone(), symbol.clone());
        self.units.insert(symbol, def);
    }

    fn register_all(&mut self) {
        self.register_length_units();
        self.register_mass_units();
        self.register_time_units();
        self.register_temperature_units();
        self.register_angle_units();
        self.register_data_units();
        self.register_area_units();
        self.register_volume_units();
        self.register_speed_units();
        self.register_force_units();
        self.register_energy_units();
        self.register_power_units();
        self.register_pressure_units();
        self.register_frequency_units();
    }

    fn register_length_units(&mut self) {
        let dim = Dimension::base(BaseQuantity::Length, 1);
        let units = vec![
            ("m", "meter", 1.0, &["meter", "meters"] as &[&str]),
            ("km", "kilometer", 1000.0, &["kilometer", "kilometers"]),
            ("cm", "centimeter", 0.01, &["centimeter", "centimeters"]),
            ("mm", "millimeter", 0.001, &["millimeter", "millimeters"]),
            (
                "um",
                "micrometer",
                1e-6,
                &["micrometer", "micron", "microns"],
            ),
            ("nm", "nanometer", 1e-9, &["nanometer", "nanometers"]),
            ("ft", "foot", 0.3048, &["foot", "feet"]),
            ("in", "inch", 0.0254, &["inch", "inches"]),
            ("yd", "yard", 0.9144, &["yard", "yards"]),
            ("mi", "mile", 1609.344, &["mile", "miles"]),
            (
                "nmi",
                "nautical mile",
                1852.0,
                &["nautical_mile", "nauticalmile"],
            ),
            ("ly", "light year", 9.4607e15, &["lightyear", "light_year"]),
            (
                "au",
                "astronomical unit",
                1.495978707e11,
                &["astronomical_unit"],
            ),
        ];

        for (sym, name, factor, aliases) in units {
            self.register_with_aliases(
                UnitDef {
                    symbol: sym.to_string(),
                    name: name.to_string(),
                    dimension: dim.clone(),
                    factor,
                    offset: 0.0,
                    category: "length".to_string(),
                },
                aliases,
            );
        }
    }

    fn register_mass_units(&mut self) {
        let dim = Dimension::base(BaseQuantity::Mass, 1);
        let units = vec![
            (
                "kg",
                "kilogram",
                1.0,
                &["kilogram", "kilograms", "kilo"] as &[&str],
            ),
            ("g", "gram", 0.001, &["gram", "grams"]),
            ("mg", "milligram", 1e-6, &["milligram", "milligrams"]),
            ("t", "metric ton", 1000.0, &["ton", "tonne", "metric_ton"]),
            ("lb", "pound", 0.45359237, &["pound", "pounds", "lbs"]),
            ("oz", "ounce", 0.028349523125, &["ounce", "ounces"]),
            ("st", "stone", 6.35029318, &["stone", "stones"]),
            ("uston", "US ton", 907.18474, &["us_ton", "short_ton"]),
        ];

        for (sym, name, factor, aliases) in units {
            self.register_with_aliases(
                UnitDef {
                    symbol: sym.to_string(),
                    name: name.to_string(),
                    dimension: dim.clone(),
                    factor,
                    offset: 0.0,
                    category: "mass".to_string(),
                },
                aliases,
            );
        }
    }

    fn register_time_units(&mut self) {
        let dim = Dimension::base(BaseQuantity::Time, 1);
        let units = vec![
            (
                "s",
                "second",
                1.0,
                &["second", "seconds", "sec", "secs"] as &[&str],
            ),
            ("ms", "millisecond", 0.001, &["millisecond", "milliseconds"]),
            ("us", "microsecond", 1e-6, &["microsecond", "microseconds"]),
            ("ns", "nanosecond", 1e-9, &["nanosecond", "nanoseconds"]),
            ("min", "minute", 60.0, &["minute", "minutes"]),
            ("h", "hour", 3600.0, &["hour", "hours", "hr", "hrs"]),
            ("d", "day", 86400.0, &["day", "days"]),
            ("week", "week", 604800.0, &["weeks"]),
            ("yr", "year", 31557600.0, &["year", "years"]),
        ];

        for (sym, name, factor, aliases) in units {
            self.register_with_aliases(
                UnitDef {
                    symbol: sym.to_string(),
                    name: name.to_string(),
                    dimension: dim.clone(),
                    factor,
                    offset: 0.0,
                    category: "time".to_string(),
                },
                aliases,
            );
        }
    }

    fn register_temperature_units(&mut self) {
        let dim = Dimension::base(BaseQuantity::Temperature, 1);

        // Kelvin is the base unit (factor=1, offset=0)
        self.register(UnitDef {
            symbol: "K".to_string(),
            name: "kelvin".to_string(),
            dimension: dim.clone(),
            factor: 1.0,
            offset: 0.0,
            category: "temperature".to_string(),
        });

        // Celsius: K = C + 273.15, so factor=1, offset=273.15
        self.register_with_aliases(
            UnitDef {
                symbol: "C".to_string(),
                name: "celsius".to_string(),
                dimension: dim.clone(),
                factor: 1.0,
                offset: 273.15,
                category: "temperature".to_string(),
            },
            &["celsius", "degC", "deg_C", "degc"],
        );

        // Fahrenheit: K = (F - 32) * 5/9 + 273.15
        // In our system: value_si = value * factor + offset
        // So: K = F * (5/9) + (273.15 - 32 * 5/9) = F * (5/9) + 255.372...
        self.register_with_aliases(
            UnitDef {
                symbol: "F".to_string(),
                name: "fahrenheit".to_string(),
                dimension: dim.clone(),
                factor: 5.0 / 9.0,
                offset: 273.15 - 32.0 * 5.0 / 9.0,
                category: "temperature".to_string(),
            },
            &["fahrenheit", "degF", "deg_F", "degf"],
        );

        // Rankine: K = R * 5/9
        self.register_with_aliases(
            UnitDef {
                symbol: "R".to_string(),
                name: "rankine".to_string(),
                dimension: dim,
                factor: 5.0 / 9.0,
                offset: 0.0,
                category: "temperature".to_string(),
            },
            &["rankine", "degR", "deg_R", "degr"],
        );
    }

    fn register_angle_units(&mut self) {
        let dim = Dimension::base(BaseQuantity::Angle, 1);
        let units = vec![
            ("rad", "radian", 1.0, &["radian", "radians"] as &[&str]),
            (
                "deg",
                "degree",
                std::f64::consts::PI / 180.0,
                &["degree", "degrees"],
            ),
            (
                "grad",
                "gradian",
                std::f64::consts::PI / 200.0,
                &["gradian", "gon"],
            ),
            (
                "arcmin",
                "arcminute",
                std::f64::consts::PI / 10800.0,
                &["arcminute", "moa"],
            ),
            (
                "arcsec",
                "arcsecond",
                std::f64::consts::PI / 648000.0,
                &["arcsecond", "soa"],
            ),
        ];

        for (sym, name, factor, aliases) in units {
            self.register_with_aliases(
                UnitDef {
                    symbol: sym.to_string(),
                    name: name.to_string(),
                    dimension: dim.clone(),
                    factor,
                    offset: 0.0,
                    category: "angle".to_string(),
                },
                aliases,
            );
        }
    }

    fn register_data_units(&mut self) {
        let dim = Dimension::base(BaseQuantity::Data, 1);
        let units = vec![
            ("bit", "bit", 1.0, &[] as &[&str]),
            ("byte", "byte", 8.0, &["Byte", "bytes"]),
            ("KB", "kilobyte", 8.0 * 1000.0, &["kilobyte", "kilobytes"]),
            ("MB", "megabyte", 8.0 * 1e6, &["megabyte", "megabytes"]),
            ("GB", "gigabyte", 8.0 * 1e9, &["gigabyte", "gigabytes"]),
            ("TB", "terabyte", 8.0 * 1e12, &["terabyte", "terabytes"]),
            ("KiB", "kibibyte", 8.0 * 1024.0, &["kibibyte"]),
            ("MiB", "mebibyte", 8.0 * 1024.0 * 1024.0, &["mebibyte"]),
            (
                "GiB",
                "gibibyte",
                8.0 * 1024.0 * 1024.0 * 1024.0,
                &["gibibyte"],
            ),
            ("TiB", "tebibyte", 8.0 * 1024f64.powi(4), &["tebibyte"]),
        ];

        for (sym, name, factor, aliases) in units {
            self.register_with_aliases(
                UnitDef {
                    symbol: sym.to_string(),
                    name: name.to_string(),
                    dimension: dim.clone(),
                    factor,
                    offset: 0.0,
                    category: "data".to_string(),
                },
                aliases,
            );
        }
    }

    fn register_area_units(&mut self) {
        let dim = Dimension::base(BaseQuantity::Length, 2);
        let units = vec![
            (
                "m2",
                "square meter",
                1.0,
                &["sq_m", "square_meter"] as &[&str],
            ),
            (
                "km2",
                "square kilometer",
                1e6,
                &["sq_km", "square_kilometer"],
            ),
            ("cm2", "square centimeter", 1e-4, &["sq_cm"]),
            ("ha", "hectare", 1e4, &["hectare"]),
            ("ac", "acre", 4046.8564224, &["acre", "acres"]),
            ("ft2", "square foot", 0.09290304, &["sq_ft", "square_foot"]),
            ("in2", "square inch", 0.00064516, &["sq_in"]),
            ("mi2", "square mile", 2589988.110336, &["sq_mi"]),
        ];

        for (sym, name, factor, aliases) in units {
            self.register_with_aliases(
                UnitDef {
                    symbol: sym.to_string(),
                    name: name.to_string(),
                    dimension: dim.clone(),
                    factor,
                    offset: 0.0,
                    category: "area".to_string(),
                },
                aliases,
            );
        }
    }

    fn register_volume_units(&mut self) {
        let dim = Dimension::base(BaseQuantity::Length, 3);
        let units = vec![
            (
                "m3",
                "cubic meter",
                1.0,
                &["cubic_meter", "cu_m"] as &[&str],
            ),
            ("L", "liter", 0.001, &["liter", "liters", "litre", "litres"]),
            ("mL", "milliliter", 1e-6, &["milliliter", "millilitre"]),
            (
                "gal",
                "gallon",
                0.003785411784,
                &["gallon", "gallons", "us_gallon"],
            ),
            ("qt", "quart", 0.000946352946, &["quart", "quarts"]),
            ("pt", "pint", 0.000473176473, &["pint", "pints"]),
            ("cup", "cup", 0.0002365882365, &["cups"]),
            (
                "floz",
                "fluid ounce",
                2.95735296e-5,
                &["fluid_ounce", "fl_oz"],
            ),
            (
                "ft3",
                "cubic foot",
                0.028316846592,
                &["cubic_foot", "cu_ft"],
            ),
            ("in3", "cubic inch", 1.6387064e-5, &["cubic_inch", "cu_in"]),
        ];

        for (sym, name, factor, aliases) in units {
            self.register_with_aliases(
                UnitDef {
                    symbol: sym.to_string(),
                    name: name.to_string(),
                    dimension: dim.clone(),
                    factor,
                    offset: 0.0,
                    category: "volume".to_string(),
                },
                aliases,
            );
        }
    }

    fn register_speed_units(&mut self) {
        // Speed = Length / Time
        let dim = Dimension::base(BaseQuantity::Length, 1)
            .divide(&Dimension::base(BaseQuantity::Time, 1));
        let units = vec![
            (
                "mps",
                "meter per second",
                1.0,
                &["m/s", "m_per_s"] as &[&str],
            ),
            (
                "kmh",
                "kilometer per hour",
                1000.0 / 3600.0,
                &["km/h", "kph", "km_per_h"],
            ),
            (
                "mph",
                "mile per hour",
                1609.344 / 3600.0,
                &["mi/h", "mi_per_h"],
            ),
            ("knot", "knot", 1852.0 / 3600.0, &["kn", "knots"]),
            ("fps", "foot per second", 0.3048, &["ft/s", "ft_per_s"]),
        ];

        for (sym, name, factor, aliases) in units {
            self.register_with_aliases(
                UnitDef {
                    symbol: sym.to_string(),
                    name: name.to_string(),
                    dimension: dim.clone(),
                    factor,
                    offset: 0.0,
                    category: "speed".to_string(),
                },
                aliases,
            );
        }
    }

    fn register_force_units(&mut self) {
        // Force = Mass * Length / Time^2 (Newton)
        let dim = Dimension::base(BaseQuantity::Mass, 1)
            .multiply(&Dimension::base(BaseQuantity::Length, 1))
            .divide(&Dimension::base(BaseQuantity::Time, 2));
        let units = vec![
            ("N", "newton", 1.0, &["newton", "newtons"] as &[&str]),
            ("kN", "kilonewton", 1000.0, &["kilonewton", "kilonewtons"]),
            ("lbf", "pound-force", 4.4482216152605, &["pound_force"]),
            ("dyn", "dyne", 1e-5, &["dyne", "dynes"]),
            (
                "kgf",
                "kilogram-force",
                9.80665,
                &["kilogram_force", "kilopond"],
            ),
        ];

        for (sym, name, factor, aliases) in units {
            self.register_with_aliases(
                UnitDef {
                    symbol: sym.to_string(),
                    name: name.to_string(),
                    dimension: dim.clone(),
                    factor,
                    offset: 0.0,
                    category: "force".to_string(),
                },
                aliases,
            );
        }
    }

    fn register_energy_units(&mut self) {
        // Energy = Mass * Length^2 / Time^2 (Joule)
        let dim = Dimension::base(BaseQuantity::Mass, 1)
            .multiply(&Dimension::base(BaseQuantity::Length, 2))
            .divide(&Dimension::base(BaseQuantity::Time, 2));
        let units = vec![
            ("J", "joule", 1.0, &["joule", "joules"] as &[&str]),
            ("kJ", "kilojoule", 1000.0, &["kilojoule"]),
            ("cal", "calorie", 4.184, &["calorie", "calories"]),
            (
                "kcal",
                "kilocalorie",
                4184.0,
                &["kilocalorie", "Cal", "Calorie"],
            ),
            ("Wh", "watt-hour", 3600.0, &["watt_hour", "watthour"]),
            (
                "kWh",
                "kilowatt-hour",
                3.6e6,
                &["kilowatt_hour", "kilowatthour"],
            ),
            (
                "BTU",
                "british thermal unit",
                1055.05585262,
                &["btu", "british_thermal_unit"],
            ),
            (
                "eV",
                "electronvolt",
                1.602176634e-19,
                &["electronvolt", "electron_volt"],
            ),
            ("erg", "erg", 1e-7, &["ergs"]),
        ];

        for (sym, name, factor, aliases) in units {
            self.register_with_aliases(
                UnitDef {
                    symbol: sym.to_string(),
                    name: name.to_string(),
                    dimension: dim.clone(),
                    factor,
                    offset: 0.0,
                    category: "energy".to_string(),
                },
                aliases,
            );
        }
    }

    fn register_power_units(&mut self) {
        // Power = Mass * Length^2 / Time^3 (Watt)
        let dim = Dimension::base(BaseQuantity::Mass, 1)
            .multiply(&Dimension::base(BaseQuantity::Length, 2))
            .divide(&Dimension::base(BaseQuantity::Time, 3));
        let units = vec![
            ("W", "watt", 1.0, &["watt", "watts"] as &[&str]),
            ("kW", "kilowatt", 1000.0, &["kilowatt"]),
            ("MW", "megawatt", 1e6, &["megawatt"]),
            ("hp", "horsepower", 745.6998715822702, &["horsepower"]),
            (
                "BTUh",
                "BTU per hour",
                1055.05585262 / 3600.0,
                &["btu_per_hour", "BTU_h"],
            ),
        ];

        for (sym, name, factor, aliases) in units {
            self.register_with_aliases(
                UnitDef {
                    symbol: sym.to_string(),
                    name: name.to_string(),
                    dimension: dim.clone(),
                    factor,
                    offset: 0.0,
                    category: "power".to_string(),
                },
                aliases,
            );
        }
    }

    fn register_pressure_units(&mut self) {
        // Pressure = Mass / (Length * Time^2) (Pascal)
        let dim = Dimension::base(BaseQuantity::Mass, 1)
            .divide(&Dimension::base(BaseQuantity::Length, 1))
            .divide(&Dimension::base(BaseQuantity::Time, 2));
        let units = vec![
            ("Pa", "pascal", 1.0, &["pascal", "pascals"] as &[&str]),
            ("kPa", "kilopascal", 1000.0, &["kilopascal"]),
            ("MPa", "megapascal", 1e6, &["megapascal"]),
            ("bar", "bar", 1e5, &["bars"]),
            (
                "atm",
                "atmosphere",
                101325.0,
                &["atmosphere", "atmospheres"],
            ),
            (
                "psi",
                "pound per square inch",
                6894.757293168,
                &["pound_per_square_inch"],
            ),
            ("torr", "torr", 133.322368421, &["Torr"]),
            (
                "mmHg",
                "millimeter of mercury",
                133.322387415,
                &["millimeter_mercury"],
            ),
        ];

        for (sym, name, factor, aliases) in units {
            self.register_with_aliases(
                UnitDef {
                    symbol: sym.to_string(),
                    name: name.to_string(),
                    dimension: dim.clone(),
                    factor,
                    offset: 0.0,
                    category: "pressure".to_string(),
                },
                aliases,
            );
        }
    }

    fn register_frequency_units(&mut self) {
        // Frequency = 1 / Time (Hertz)
        let dim = Dimension::base(BaseQuantity::Time, -1);
        let units = vec![
            ("Hz", "hertz", 1.0, &["hertz"] as &[&str]),
            ("kHz", "kilohertz", 1000.0, &["kilohertz"]),
            ("MHz", "megahertz", 1e6, &["megahertz"]),
            ("GHz", "gigahertz", 1e9, &["gigahertz"]),
        ];

        for (sym, name, factor, aliases) in units {
            self.register_with_aliases(
                UnitDef {
                    symbol: sym.to_string(),
                    name: name.to_string(),
                    dimension: dim.clone(),
                    factor,
                    offset: 0.0,
                    category: "frequency".to_string(),
                },
                aliases,
            );
        }
    }
}

impl Default for UnitRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dimension_display() {
        assert_eq!(Dimension::dimensionless().to_string(), "");
        assert_eq!(Dimension::base(BaseQuantity::Length, 1).to_string(), "m");
        assert_eq!(Dimension::base(BaseQuantity::Time, -1).to_string(), "/s");
    }

    #[test]
    fn test_dimension_multiply() {
        let speed = Dimension::base(BaseQuantity::Length, 1)
            .divide(&Dimension::base(BaseQuantity::Time, 1));
        assert_eq!(speed.exponents, [1, 0, -1, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn test_convert_length() {
        let reg = UnitRegistry::new();
        // 1 meter = 3.28084 feet
        let result = reg.convert(1.0, "m", "ft").unwrap();
        assert!((result - 3.28083989501).abs() < 1e-6);
    }

    #[test]
    fn test_convert_temperature() {
        let reg = UnitRegistry::new();
        // 0 C = 32 F
        let result = reg.convert(0.0, "C", "F").unwrap();
        assert!((result - 32.0).abs() < 1e-6);

        // 100 C = 212 F
        let result = reg.convert(100.0, "C", "F").unwrap();
        assert!((result - 212.0).abs() < 1e-6);

        // 0 C = 273.15 K
        let result = reg.convert(0.0, "C", "K").unwrap();
        assert!((result - 273.15).abs() < 1e-6);
    }

    #[test]
    fn test_convert_incompatible() {
        let reg = UnitRegistry::new();
        let result = reg.convert(1.0, "m", "kg");
        assert!(result.is_err());
    }

    #[test]
    fn test_unknown_unit() {
        let reg = UnitRegistry::new();
        let result = reg.convert(1.0, "foo", "bar");
        assert!(result.is_err());
    }

    #[test]
    fn test_unit_aliases() {
        let reg = UnitRegistry::new();
        assert!(reg.get("meter").is_some());
        assert!(reg.get("m").is_some());
        assert!(reg.get("feet").is_some());
        assert!(reg.get("ft").is_some());
    }

    #[test]
    fn test_to_base_and_from_base() {
        let reg = UnitRegistry::new();
        let (value, dim) = reg.to_base(1.0, "ft").unwrap();
        assert!((value - 0.3048).abs() < 1e-10);
        assert_eq!(dim, Dimension::base(BaseQuantity::Length, 1));

        let result = reg.from_base(0.3048, &dim, "ft").unwrap();
        assert!((result - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_categories() {
        let reg = UnitRegistry::new();
        let cats = reg.categories();
        assert!(cats.contains(&"length".to_string()));
        assert!(cats.contains(&"mass".to_string()));
        assert!(cats.contains(&"temperature".to_string()));
        assert!(cats.contains(&"time".to_string()));
    }

    #[test]
    fn test_data_units() {
        let reg = UnitRegistry::new();
        // 1 byte = 8 bits
        let result = reg.convert(1.0, "byte", "bit").unwrap();
        assert!((result - 8.0).abs() < 1e-10);

        // 1 KB = 1000 bytes
        let result = reg.convert(1.0, "KB", "byte").unwrap();
        assert!((result - 1000.0).abs() < 1e-6);
    }

    #[test]
    fn test_pressure_units() {
        let reg = UnitRegistry::new();
        // 1 atm = 101325 Pa
        let result = reg.convert(1.0, "atm", "Pa").unwrap();
        assert!((result - 101325.0).abs() < 1e-6);
    }

    #[test]
    fn test_energy_units() {
        let reg = UnitRegistry::new();
        // 1 kWh = 3.6e6 J
        let result = reg.convert(1.0, "kWh", "J").unwrap();
        assert!((result - 3.6e6).abs() < 1e-6);
    }
}
