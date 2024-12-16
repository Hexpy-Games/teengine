pub const SPRITE_VERTEX_SHADER: &str = r#"
    #version 330 core
    layout (location = 0) in vec2 aPos;
    layout (location = 1) in vec2 aTexCoords;

    out vec2 TexCoords;

    uniform mat4 model;
    uniform mat4 projection;

    void main()
    {
        TexCoords = aTexCoords;
        gl_Position = projection * model * vec4(aPos, 0.0, 1.0);
    }
"#;

pub const SPRITE_FRAGMENT_SHADER: &str = r#"
    #version 330 core
    in vec2 TexCoords;
    out vec4 FragColor;

    uniform sampler2D image;
    uniform vec3 colorKey;
    uniform bool useColorKey;
    uniform float threshold;

    void main()
    {
        vec4 texColor = texture(image, TexCoords);

        if (useColorKey) {
            vec3 diff = abs(texColor.rgb - colorKey);
            float maxDiff = max(max(diff.r, diff.g), diff.b);

            if (maxDiff < threshold) {
                discard;
            }
        }

        FragColor = texColor;
    }
"#;
