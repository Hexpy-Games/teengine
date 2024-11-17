use glam::Vec3;

#[derive(Debug, Clone, Copy)]
pub struct ColorKey {
    pub color: Vec3,
    pub threshold: f32,
}

impl ColorKey {
    /// Generate ColorKey from hex code
    ///
    /// # Arguments
    /// * `hex` - Hex color code (e.g. "#FFFFFF" or "FFFFFF")
    /// * `threshold` - Threshold value
    ///
    /// # Returns
    /// * `Result<Self, String>` - ColorKey or error message
    ///
    /// # Examples
    /// ```
    /// let color_key = ColorKey::from_hex("#FFFFFF", 0.5).unwrap();
    /// ```
    #[allow(unused)]
    pub fn from_hex(
        hex: &str,
        threshold: f32,
    ) -> Result<Self, String> {
        let hex = hex.trim_start_matches('#');

        if hex.len() != 6 {
            return Err("Invalid hex color code length".to_string());
        }

        // convert hex string to u8 bytes
        let r = u8::from_str_radix(&hex[0..2], 16)
            .map_err(|_| "Invalid red component")?;
        let g = u8::from_str_radix(&hex[2..4], 16)
            .map_err(|_| "Invalid green component")?;
        let b = u8::from_str_radix(&hex[4..6], 16)
            .map_err(|_| "Invalid blue component")?;

        Ok(Self {
            color: Vec3::new(
                r as f32 / 255.0,
                g as f32 / 255.0,
                b as f32 / 255.0,
            ),
            threshold,
        })
    }

    /// Create ColorKey from RGB values (0-255 range)
    ///
    /// # Arguments
    /// * `r` - Red value
    /// * `g` - Green value
    /// * `b` - Blue value
    /// * `threshold` - Threshold value
    ///
    /// # Returns
    /// * `Result<Self, String>` - ColorKey or error message
    ///
    /// # Examples
    /// ```
    /// let color_key = ColorKey::from_rgb(255, 255, 255, 0.5).unwrap();
    /// ```
    #[allow(unused)]
    pub fn from_rgb(
        r: u8,
        g: u8,
        b: u8,
        threshold: f32,
    ) -> Result<Self, String> {
        Ok(Self {
            color: Vec3::new(
                r as f32 / 255.0,
                g as f32 / 255.0,
                b as f32 / 255.0,
            ),
            threshold,
        })
    }

    /// Return hex string for debugging
    #[cfg(test)]
    #[allow(unused)]
    pub fn to_hex(&self) -> String {
        let r = (self.color.x * 255.0) as u8;
        let g = (self.color.y * 255.0) as u8;
        let b = (self.color.z * 255.0) as u8;
        format!("#{:02X}{:02X}{:02X}", r, g, b)
    }
}
