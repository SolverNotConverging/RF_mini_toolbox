#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::f64::consts::PI;

use num_complex::Complex64;
use slint::SharedString;

slint::include_modules!();

const C0: f64 = 299_792_458.0;
const EPSILON: f64 = 1.0e-12;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MatrixKind {
    Abcd,
    Z,
    Y,
    S,
}

impl MatrixKind {
    fn from_index(index: i32) -> Self {
        match index {
            0 => Self::Abcd,
            1 => Self::Z,
            2 => Self::Y,
            3 => Self::S,
            _ => Self::Abcd,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Abcd => "ABCD",
            Self::Z => "Z",
            Self::Y => "Y",
            Self::S => "S",
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Matrix2 {
    a11: Complex64,
    a12: Complex64,
    a21: Complex64,
    a22: Complex64,
}

impl Matrix2 {
    fn new(a11: Complex64, a12: Complex64, a21: Complex64, a22: Complex64) -> Self {
        Self { a11, a12, a21, a22 }
    }

    fn identity() -> Self {
        Self::new(
            Complex64::new(1.0, 0.0),
            Complex64::new(0.0, 0.0),
            Complex64::new(0.0, 0.0),
            Complex64::new(1.0, 0.0),
        )
    }

    fn determinant(self) -> Complex64 {
        self.a11 * self.a22 - self.a12 * self.a21
    }

    fn inverse(self) -> Result<Self, String> {
        let det = self.determinant();
        if near_zero(det) {
            return Err(
                "Matrix inversion failed because the determinant is too close to zero.".into(),
            );
        }

        Ok(Self::new(
            self.a22 / det,
            -self.a12 / det,
            -self.a21 / det,
            self.a11 / det,
        ))
    }

    fn add(self, other: Self) -> Self {
        Self::new(
            self.a11 + other.a11,
            self.a12 + other.a12,
            self.a21 + other.a21,
            self.a22 + other.a22,
        )
    }

    fn sub(self, other: Self) -> Self {
        Self::new(
            self.a11 - other.a11,
            self.a12 - other.a12,
            self.a21 - other.a21,
            self.a22 - other.a22,
        )
    }

    fn mul(self, other: Self) -> Self {
        Self::new(
            self.a11 * other.a11 + self.a12 * other.a21,
            self.a11 * other.a12 + self.a12 * other.a22,
            self.a21 * other.a11 + self.a22 * other.a21,
            self.a21 * other.a12 + self.a22 * other.a22,
        )
    }

    fn scale(self, value: Complex64) -> Self {
        Self::new(
            self.a11 * value,
            self.a12 * value,
            self.a21 * value,
            self.a22 * value,
        )
    }
}

fn main() -> Result<(), slint::PlatformError> {
    let app = MainWindow::new()?;

    {
        let weak = app.as_weak();
        app.on_freq_to_lambda(move || {
            if let Some(app) = weak.upgrade() {
                let status = match convert_freq_to_lambda(&app) {
                    Ok(message) => message,
                    Err(message) => message,
                };
                app.set_status_text(status.into());
            }
        });
    }

    {
        let weak = app.as_weak();
        app.on_lambda_to_freq(move || {
            if let Some(app) = weak.upgrade() {
                let status = match convert_lambda_to_freq(&app) {
                    Ok(message) => message,
                    Err(message) => message,
                };
                app.set_status_text(status.into());
            }
        });
    }

    {
        let weak = app.as_weak();
        app.on_convert_matrix(move || {
            if let Some(app) = weak.upgrade() {
                let status = match convert_matrix(&app) {
                    Ok(message) => message,
                    Err(message) => message,
                };
                app.set_status_text(status.into());
            }
        });
    }

    {
        let weak = app.as_weak();
        app.on_matrix_use_output(move || {
            if let Some(app) = weak.upgrade() {
                promote_matrix_output(&app);
                app.set_status_text("Output matrix copied back into the input section.".into());
            }
        });
    }

    {
        let weak = app.as_weak();
        app.on_component_to_reactance(move || {
            if let Some(app) = weak.upgrade() {
                let status = match convert_component_to_reactance(&app) {
                    Ok(message) => message,
                    Err(message) => message,
                };
                app.set_status_text(status.into());
            }
        });
    }

    {
        let weak = app.as_weak();
        app.on_reactance_to_component(move || {
            if let Some(app) = weak.upgrade() {
                let status = match convert_reactance_to_component(&app) {
                    Ok(message) => message,
                    Err(message) => message,
                };
                app.set_status_text(status.into());
            }
        });
    }

    {
        let weak = app.as_weak();
        app.on_solve_microstrip(move || {
            if let Some(app) = weak.upgrade() {
                let status = match solve_microstrip(&app) {
                    Ok(message) => message,
                    Err(message) => message,
                };
                app.set_status_text(status.into());
            }
        });
    }

    app.run()
}

fn convert_freq_to_lambda(app: &MainWindow) -> Result<String, String> {
    let eps_r = parse_positive("Relative permittivity", app.get_wl_eps_r())?;
    let mu_r = parse_positive("Relative permeability", app.get_wl_mu_r())?;
    let freq_value = parse_positive("Frequency", app.get_wl_freq_value())?;

    let freq_hz = freq_value * general_frequency_scale(app.get_wl_freq_unit());
    let wavelength_m = C0 / (freq_hz * (eps_r * mu_r).sqrt());
    let display_value = wavelength_m / wavelength_scale(app.get_wl_lambda_unit());

    app.set_wl_eps_r(format_number(eps_r).into());
    app.set_wl_mu_r(format_number(mu_r).into());
    app.set_wl_freq_value(format_number(freq_value).into());
    app.set_wl_lambda_value(format_number(display_value).into());

    Ok(format!(
        "Converted frequency to wavelength: {:.6} m in the selected medium.",
        wavelength_m
    ))
}

fn convert_lambda_to_freq(app: &MainWindow) -> Result<String, String> {
    let eps_r = parse_positive("Relative permittivity", app.get_wl_eps_r())?;
    let mu_r = parse_positive("Relative permeability", app.get_wl_mu_r())?;
    let lambda_value = parse_positive("Wavelength", app.get_wl_lambda_value())?;

    let wavelength_m = lambda_value * wavelength_scale(app.get_wl_lambda_unit());
    let freq_hz = C0 / (wavelength_m * (eps_r * mu_r).sqrt());
    let display_value = freq_hz / general_frequency_scale(app.get_wl_freq_unit());

    app.set_wl_eps_r(format_number(eps_r).into());
    app.set_wl_mu_r(format_number(mu_r).into());
    app.set_wl_freq_value(format_number(display_value).into());
    app.set_wl_lambda_value(format_number(lambda_value).into());

    Ok(format!(
        "Converted wavelength to frequency: {:.6} Hz in the selected medium.",
        freq_hz
    ))
}

fn convert_matrix(app: &MainWindow) -> Result<String, String> {
    let from = MatrixKind::from_index(app.get_matrix_from_type());
    let to = MatrixKind::from_index(app.get_matrix_to_type());
    let z0 = parse_positive("Reference impedance Z0", app.get_matrix_z0())?;
    let input = read_matrix_input(app)?;

    let output = if from == to {
        input
    } else {
        transform_matrix(input, from, to, z0)?
    };

    write_matrix_output(app, output);

    Ok(format!(
        "Converted {} matrix to {} matrix using Z0 = {} ohm.",
        from.label(),
        to.label(),
        format_number(z0)
    ))
}

fn promote_matrix_output(app: &MainWindow) {
    app.set_matrix_from_type(app.get_matrix_to_type());
    app.set_mx_in_11_re(app.get_mx_out_11_re());
    app.set_mx_in_11_im(app.get_mx_out_11_im());
    app.set_mx_in_12_re(app.get_mx_out_12_re());
    app.set_mx_in_12_im(app.get_mx_out_12_im());
    app.set_mx_in_21_re(app.get_mx_out_21_re());
    app.set_mx_in_21_im(app.get_mx_out_21_im());
    app.set_mx_in_22_re(app.get_mx_out_22_re());
    app.set_mx_in_22_im(app.get_mx_out_22_im());
}

fn convert_component_to_reactance(app: &MainWindow) -> Result<String, String> {
    let frequency = parse_positive("Frequency", app.get_react_freq_value())?
        * react_frequency_scale(app.get_react_freq_unit());
    let react_kind = app.get_react_kind();
    let reactance_ohms = if react_kind == 0 {
        let inductance = parse_positive("Inductance", app.get_react_inductance_value())?
            * inductance_scale(app.get_react_inductance_unit());
        let reactance = 2.0 * PI * frequency * inductance;
        app.set_react_inductance_value(
            format_number(inductance / inductance_scale(app.get_react_inductance_unit())).into(),
        );
        reactance
    } else {
        let capacitance = parse_positive("Capacitance", app.get_react_capacitance_value())?
            * capacitance_scale(app.get_react_capacitance_unit());
        let reactance = 1.0 / (2.0 * PI * frequency * capacitance);
        app.set_react_capacitance_value(
            format_number(capacitance / capacitance_scale(app.get_react_capacitance_unit())).into(),
        );
        reactance
    };

    let display_reactance = reactance_ohms / reactance_scale(app.get_react_x_unit());
    app.set_react_x_value(format_number(display_reactance).into());
    app.set_react_freq_value(
        format_number(frequency / react_frequency_scale(app.get_react_freq_unit())).into(),
    );
    app.set_react_impedance_text(format_impedance(reactance_ohms, react_kind == 0).into());

    Ok(if react_kind == 0 {
        format!(
            "Calculated inductive reactance: {}.",
            format_impedance(reactance_ohms, true)
        )
    } else {
        format!(
            "Calculated capacitive reactance: {}.",
            format_impedance(reactance_ohms, false)
        )
    })
}

fn convert_reactance_to_component(app: &MainWindow) -> Result<String, String> {
    let frequency = parse_positive("Frequency", app.get_react_freq_value())?
        * react_frequency_scale(app.get_react_freq_unit());
    let reactance = parse_positive("Reactance magnitude", app.get_react_x_value())?
        * reactance_scale(app.get_react_x_unit());
    let react_kind = app.get_react_kind();

    if react_kind == 0 {
        let inductance = reactance / (2.0 * PI * frequency);
        app.set_react_inductance_value(
            format_number(inductance / inductance_scale(app.get_react_inductance_unit())).into(),
        );
    } else {
        let capacitance = 1.0 / (2.0 * PI * frequency * reactance);
        app.set_react_capacitance_value(
            format_number(capacitance / capacitance_scale(app.get_react_capacitance_unit())).into(),
        );
    }

    app.set_react_freq_value(
        format_number(frequency / react_frequency_scale(app.get_react_freq_unit())).into(),
    );
    app.set_react_x_value(
        format_number(reactance / reactance_scale(app.get_react_x_unit())).into(),
    );
    app.set_react_impedance_text(format_impedance(reactance, react_kind == 0).into());

    Ok(if react_kind == 0 {
        "Converted inductive reactance back to inductance.".into()
    } else {
        "Converted capacitive reactance back to capacitance.".into()
    })
}

fn solve_microstrip(app: &MainWindow) -> Result<String, String> {
    let er = parse_positive("Relative permittivity", app.get_ms_er())?;
    let width = parse_positive("Trace width", app.get_ms_width())?
        * microstrip_length_scale(app.get_ms_dimension_unit());
    let height = parse_positive("Substrate height", app.get_ms_height())?
        * microstrip_length_scale(app.get_ms_dimension_unit());
    let frequency = parse_positive("Frequency", app.get_ms_freq_value())?
        * microstrip_frequency_scale(app.get_ms_freq_unit());

    if er <= 1.0 {
        return Err(
            "Relative permittivity should be greater than 1 for a practical microstrip.".into(),
        );
    }

    let u = width / height;
    if u <= 0.0 {
        return Err("Width to height ratio must be positive.".into());
    }

    let a = 1.0
        + (1.0 / 49.0) * ((u.powi(4) + (u / 52.0).powi(2)) / (u.powi(4) + 0.432)).ln()
        + (1.0 / 18.7) * (1.0 + u / 18.1).ln();
    let b = 0.564 * ((er - 0.9) / (er + 3.0)).powf(0.053);
    let effective_er = ((er + 1.0) / 2.0) + ((er - 1.0) / 2.0) * (1.0 + 10.0 / u).powf(-a * b);

    let z0 = if u <= 1.0 {
        (60.0 / effective_er.sqrt()) * ((8.0 / u) + 0.25 * u).ln()
    } else {
        (120.0 * PI) / (effective_er.sqrt() * (u + 1.393 + 0.667 * (u + 1.444).ln()))
    };

    let guided_wavelength_m = C0 / (frequency * effective_er.sqrt());
    let lambda_display = guided_wavelength_m / microstrip_length_scale(app.get_ms_dimension_unit());

    app.set_ms_er(format_number(er).into());
    app.set_ms_width(
        format_number(width / microstrip_length_scale(app.get_ms_dimension_unit())).into(),
    );
    app.set_ms_height(
        format_number(height / microstrip_length_scale(app.get_ms_dimension_unit())).into(),
    );
    app.set_ms_freq_value(
        format_number(frequency / microstrip_frequency_scale(app.get_ms_freq_unit())).into(),
    );
    app.set_ms_z0_result(format_number(z0).into());
    app.set_ms_eeff_result(format_number(effective_er).into());
    app.set_ms_lambda_result(format_number(lambda_display).into());

    Ok(format!(
        "Calculated microstrip impedance with Hammerstad-Jensen equations: Z0 = {} ohm.",
        format_number(z0)
    ))
}

fn read_matrix_input(app: &MainWindow) -> Result<Matrix2, String> {
    Ok(Matrix2::new(
        Complex64::new(
            parse_number("M11 real", app.get_mx_in_11_re())?,
            parse_number("M11 imag", app.get_mx_in_11_im())?,
        ),
        Complex64::new(
            parse_number("M12 real", app.get_mx_in_12_re())?,
            parse_number("M12 imag", app.get_mx_in_12_im())?,
        ),
        Complex64::new(
            parse_number("M21 real", app.get_mx_in_21_re())?,
            parse_number("M21 imag", app.get_mx_in_21_im())?,
        ),
        Complex64::new(
            parse_number("M22 real", app.get_mx_in_22_re())?,
            parse_number("M22 imag", app.get_mx_in_22_im())?,
        ),
    ))
}

fn write_matrix_output(app: &MainWindow, matrix: Matrix2) {
    app.set_mx_out_11_re(format_number(matrix.a11.re).into());
    app.set_mx_out_11_im(format_number(matrix.a11.im).into());
    app.set_mx_out_12_re(format_number(matrix.a12.re).into());
    app.set_mx_out_12_im(format_number(matrix.a12.im).into());
    app.set_mx_out_21_re(format_number(matrix.a21.re).into());
    app.set_mx_out_21_im(format_number(matrix.a21.im).into());
    app.set_mx_out_22_re(format_number(matrix.a22.re).into());
    app.set_mx_out_22_im(format_number(matrix.a22.im).into());
}

fn transform_matrix(
    matrix: Matrix2,
    from: MatrixKind,
    to: MatrixKind,
    z0: f64,
) -> Result<Matrix2, String> {
    match (from, to) {
        (MatrixKind::Abcd, MatrixKind::Z) => abcd_to_z(matrix),
        (MatrixKind::Abcd, MatrixKind::Y) => abcd_to_y(matrix),
        (MatrixKind::Abcd, MatrixKind::S) => abcd_to_s(matrix, z0),
        (MatrixKind::Z, MatrixKind::Abcd) => z_to_abcd(matrix),
        (MatrixKind::Z, MatrixKind::Y) => matrix.inverse(),
        (MatrixKind::Z, MatrixKind::S) => z_to_s(matrix, z0),
        (MatrixKind::Y, MatrixKind::Abcd) => y_to_abcd(matrix),
        (MatrixKind::Y, MatrixKind::Z) => matrix.inverse(),
        (MatrixKind::Y, MatrixKind::S) => y_to_s(matrix, z0),
        (MatrixKind::S, MatrixKind::Abcd) => s_to_abcd(matrix, z0),
        (MatrixKind::S, MatrixKind::Z) => s_to_z(matrix, z0),
        (MatrixKind::S, MatrixKind::Y) => s_to_y(matrix, z0),
        _ => Ok(matrix),
    }
}

fn abcd_to_z(matrix: Matrix2) -> Result<Matrix2, String> {
    if near_zero(matrix.a21) {
        return Err("ABCD to Z conversion is singular because C is too close to zero.".into());
    }

    let det = matrix.determinant();
    Ok(Matrix2::new(
        matrix.a11 / matrix.a21,
        det / matrix.a21,
        Complex64::new(1.0, 0.0) / matrix.a21,
        matrix.a22 / matrix.a21,
    ))
}

fn z_to_abcd(matrix: Matrix2) -> Result<Matrix2, String> {
    if near_zero(matrix.a21) {
        return Err("Z to ABCD conversion is singular because Z21 is too close to zero.".into());
    }

    let det = matrix.determinant();
    Ok(Matrix2::new(
        matrix.a11 / matrix.a21,
        det / matrix.a21,
        Complex64::new(1.0, 0.0) / matrix.a21,
        matrix.a22 / matrix.a21,
    ))
}

fn abcd_to_y(matrix: Matrix2) -> Result<Matrix2, String> {
    if near_zero(matrix.a12) {
        return Err("ABCD to Y conversion is singular because B is too close to zero.".into());
    }

    let det = matrix.determinant();
    Ok(Matrix2::new(
        matrix.a22 / matrix.a12,
        -det / matrix.a12,
        -Complex64::new(1.0, 0.0) / matrix.a12,
        matrix.a11 / matrix.a12,
    ))
}

fn y_to_abcd(matrix: Matrix2) -> Result<Matrix2, String> {
    if near_zero(matrix.a21) {
        return Err("Y to ABCD conversion is singular because Y21 is too close to zero.".into());
    }

    let det = matrix.determinant();
    Ok(Matrix2::new(
        -matrix.a22 / matrix.a21,
        -Complex64::new(1.0, 0.0) / matrix.a21,
        -det / matrix.a21,
        -matrix.a11 / matrix.a21,
    ))
}

fn z_to_s(matrix: Matrix2, z0: f64) -> Result<Matrix2, String> {
    let z0c = Complex64::new(z0, 0.0);
    let identity = Matrix2::identity();
    let numerator = matrix.sub(identity.scale(z0c));
    let denominator = matrix.add(identity.scale(z0c)).inverse()?;
    Ok(numerator.mul(denominator))
}

fn s_to_z(matrix: Matrix2, z0: f64) -> Result<Matrix2, String> {
    let identity = Matrix2::identity();
    let numerator = identity.add(matrix);
    let denominator = identity.sub(matrix).inverse()?;
    Ok(numerator.mul(denominator).scale(Complex64::new(z0, 0.0)))
}

fn y_to_s(matrix: Matrix2, z0: f64) -> Result<Matrix2, String> {
    let identity = Matrix2::identity();
    let yz0 = matrix.scale(Complex64::new(z0, 0.0));
    let numerator = identity.sub(yz0);
    let denominator = identity.add(yz0).inverse()?;
    Ok(numerator.mul(denominator))
}

fn s_to_y(matrix: Matrix2, z0: f64) -> Result<Matrix2, String> {
    let identity = Matrix2::identity();
    let numerator = identity.sub(matrix);
    let denominator = identity.add(matrix).inverse()?;
    Ok(numerator
        .mul(denominator)
        .scale(Complex64::new(1.0 / z0, 0.0)))
}

fn abcd_to_s(matrix: Matrix2, z0: f64) -> Result<Matrix2, String> {
    let z0c = Complex64::new(z0, 0.0);
    let denominator = matrix.a11 + matrix.a12 / z0c + matrix.a21 * z0c + matrix.a22;
    if near_zero(denominator) {
        return Err(
            "ABCD to S conversion is singular because the denominator is too close to zero.".into(),
        );
    }

    let det = matrix.determinant();
    Ok(Matrix2::new(
        (matrix.a11 + matrix.a12 / z0c - matrix.a21 * z0c - matrix.a22) / denominator,
        (Complex64::new(2.0, 0.0) * det) / denominator,
        Complex64::new(2.0, 0.0) / denominator,
        (-matrix.a11 + matrix.a12 / z0c - matrix.a21 * z0c + matrix.a22) / denominator,
    ))
}

fn s_to_abcd(matrix: Matrix2, z0: f64) -> Result<Matrix2, String> {
    if near_zero(matrix.a21) {
        return Err("S to ABCD conversion is singular because S21 is too close to zero.".into());
    }

    let z0c = Complex64::new(z0, 0.0);
    let one = Complex64::new(1.0, 0.0);
    let two_s21 = Complex64::new(2.0, 0.0) * matrix.a21;

    Ok(Matrix2::new(
        ((one + matrix.a11) * (one - matrix.a22) + matrix.a12 * matrix.a21) / two_s21,
        z0c * (((one + matrix.a11) * (one + matrix.a22) - matrix.a12 * matrix.a21) / two_s21),
        (((one - matrix.a11) * (one - matrix.a22) - matrix.a12 * matrix.a21) / two_s21) / z0c,
        ((one - matrix.a11) * (one + matrix.a22) + matrix.a12 * matrix.a21) / two_s21,
    ))
}

fn general_frequency_scale(index: i32) -> f64 {
    match index {
        0 => 1.0,
        1 => 1.0e3,
        2 => 1.0e6,
        3 => 1.0e9,
        4 => 1.0e12,
        _ => 1.0,
    }
}

fn wavelength_scale(index: i32) -> f64 {
    match index {
        0 => 1.0e-6,
        1 => 1.0e-3,
        2 => 1.0e-2,
        3 => 1.0,
        _ => 1.0e-3,
    }
}

fn react_frequency_scale(index: i32) -> f64 {
    match index {
        0 => 1.0e6,
        1 => 1.0e9,
        2 => 1.0e12,
        _ => 1.0e9,
    }
}

fn inductance_scale(index: i32) -> f64 {
    match index {
        0 => 1.0,
        1 => 1.0e-3,
        2 => 1.0e-6,
        3 => 1.0e-9,
        4 => 1.0e-12,
        _ => 1.0e-9,
    }
}

fn capacitance_scale(index: i32) -> f64 {
    match index {
        0 => 1.0,
        1 => 1.0e-3,
        2 => 1.0e-6,
        3 => 1.0e-9,
        4 => 1.0e-12,
        _ => 1.0e-12,
    }
}

fn reactance_scale(index: i32) -> f64 {
    match index {
        0 => 1.0,
        1 => 1.0e3,
        2 => 1.0e6,
        _ => 1.0,
    }
}

fn microstrip_length_scale(index: i32) -> f64 {
    match index {
        0 => 1.0e-6,
        1 => 25.4e-6,
        2 => 1.0e-3,
        3 => 1.0e-2,
        4 => 0.0254,
        _ => 1.0e-3,
    }
}

fn microstrip_frequency_scale(index: i32) -> f64 {
    match index {
        0 => 1.0e6,
        1 => 1.0e9,
        2 => 1.0e12,
        _ => 1.0e9,
    }
}

fn parse_positive(label: &str, value: SharedString) -> Result<f64, String> {
    let parsed = parse_number(label, value)?;
    if parsed <= 0.0 {
        return Err(format!("{label} must be greater than zero."));
    }
    Ok(parsed)
}

fn parse_number(label: &str, value: SharedString) -> Result<f64, String> {
    value
        .trim()
        .parse::<f64>()
        .map_err(|_| format!("Could not parse {label}."))
}

fn near_zero(value: Complex64) -> bool {
    value.norm() < EPSILON
}

fn format_impedance(reactance_ohms: f64, inductive: bool) -> String {
    if inductive {
        format!("j{} ohm", format_number(reactance_ohms))
    } else {
        format!("-j{} ohm", format_number(reactance_ohms))
    }
}

fn format_number(value: f64) -> String {
    if !value.is_finite() {
        return "NaN".into();
    }

    if value.abs() < EPSILON {
        return "0".into();
    }

    let abs = value.abs();
    let raw = if (1.0e-3..1.0e6).contains(&abs) {
        format!("{value:.6}")
    } else {
        format!("{value:.6e}")
    };

    trim_trailing_zeros(raw)
}

fn trim_trailing_zeros(input: String) -> String {
    if let Some((base, exponent)) = input.split_once('e') {
        let trimmed = trim_decimal(base);
        format!("{trimmed}e{exponent}")
    } else {
        trim_decimal(&input)
    }
}

fn trim_decimal(input: &str) -> String {
    let mut output = input.to_string();
    while output.contains('.') && output.ends_with('0') {
        output.pop();
    }
    if output.ends_with('.') {
        output.pop();
    }
    output
}
