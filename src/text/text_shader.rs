pub const TEXT_VERTEX_SHADER: &str = r#"
    #version 330 core
    layout (location = 0) in vec2 aPos;
    layout (location = 1) in vec2 aTexCoords;

    out vec2 TexCoords;

    uniform mat4 projection;

    void main()
    {
        gl_Position = projection * vec4(aPos, 0.0, 1.0);
        TexCoords = aTexCoords;
    }
"#;

pub const TEXT_FRAGMENT_SHADER: &str = r#"
    #version 330 core
    in vec2 TexCoords;
    out vec4 FragColor;

    uniform sampler2D text;
    uniform vec4 textColor;

    void main()
    {
        vec4 sampled = texture(text, TexCoords);
        FragColor = vec4(textColor.rgb, textColor.a * sampled.a);
    }
"#;
