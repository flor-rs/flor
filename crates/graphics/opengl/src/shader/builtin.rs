// 专门放置常量或 Shader Source Code

pub const COLOR_VERTEX_SHADER: &str = r#"#version 330 core
layout (location = 0) in vec2 a_pos;

uniform mat3 u_transform;
out vec2 v_pos;

void main() {
    vec3 pos = u_transform * vec3(a_pos, 1.0);
    // 假设正交投影由 u_transform 提前算入 MVP 中
    gl_Position = vec4(pos.xy, 0.0, 1.0);
    v_pos = a_pos;
}
"#;

pub const COLOR_FRAGMENT_SHADER: &str = r#"#version 330 core
in vec2 v_pos;
out vec4 FragColor;

// 渐变统一参数
uniform int u_gradient_type; // 0: Solid, 1: Linear, 2: Radial
uniform int u_stop_count;
uniform vec2 u_start;
uniform vec2 u_end;
uniform float u_stops[32];
uniform vec4 u_colors[32];

// 超过 32 个断点的备用方案：数据纹理
uniform bool u_use_texture;
uniform sampler2D u_stop_data;

void main() {
    // 纯色
    if (u_gradient_type == 0 || u_stop_count <= 1) {
        FragColor = u_colors[0];
        return;
    }

    // 渐变投影计算 t 值
    float t = 0.0;
    if (u_gradient_type == 1) {
        // 线性渐变
        vec2 dir = u_end - u_start;
        float lenSq = dot(dir, dir);
        if (lenSq > 0.00001) {
            t = dot(v_pos - u_start, dir) / lenSq;
        }
    } else if (u_gradient_type == 2) {
        // 径向渐变
        float dist = distance(v_pos, u_start);
        float radius = u_end.x; // radius 存在 u_end.x
        if (radius > 0.00001) {
            t = dist / radius;
        }
    }
    
    t = clamp(t, 0.0, 1.0);

    vec4 color;
    
    if (u_use_texture) {
        // 使用纹理采样读取数据
        // 数据排列方式：[stop, r, g, b, a], 但这里为了简化直接让 CPU 生成 1D 颜色条（256px等宽）
        // 这样就可以利用硬件直接采样并插值。因为 256px 纹理可以直接使用 texture 采样：
        color = texture(u_stop_data, vec2(t, 0.5));
    } else {
        // Uniform 数组插值
        color = u_colors[0];
        for (int i = 0; i < u_stop_count - 1; i++) {
            if (t >= u_stops[i] && t <= u_stops[i+1]) {
                float dt = u_stops[i+1] - u_stops[i];
                float factor = 0.0;
                if (dt > 0.00001) {
                    factor = (t - u_stops[i]) / dt;
                }
                color = mix(u_colors[i], u_colors[i+1], factor);
                break;
            } else if (t > u_stops[i+1]) {
                color = u_colors[i+1];
            }
        }
    }

    // 极轻量防色带抖动 (Dithering)
    float dither = fract(sin(dot(gl_FragCoord.xy, vec2(12.9898, 78.233))) * 43758.5453) / 255.0;
    FragColor = color + vec4(dither, dither, dither, 0.0);
}
"#;

pub const TEXTURE_VERTEX_SHADER: &str = r#"#version 330 core
layout (location = 0) in vec2 a_pos;
layout (location = 1) in vec2 a_texCoord;

uniform mat3 u_transform;
out vec2 v_texCoord;
out vec2 v_pos;

void main() {
    vec3 pos = u_transform * vec3(a_pos, 1.0);
    gl_Position = vec4(pos.xy, 0.0, 1.0);
    v_texCoord = a_texCoord;
    v_pos = a_pos;
}
"#;

pub const TEXTURE_FRAGMENT_SHADER: &str = r#"#version 330 core
in vec2 v_texCoord;
out vec4 FragColor;

uniform sampler2D u_texture;
uniform float u_opacity;
uniform int u_use_tint;
uniform vec4 u_tint_color;

void main() {
    vec4 texColor = texture(u_texture, v_texCoord);
    if (u_use_tint != 0) {
        FragColor = vec4(u_tint_color.rgb, texColor.a * u_tint_color.a * u_opacity);
    } else {
        FragColor = texColor * vec4(1.0, 1.0, 1.0, u_opacity);
    }
}
"#;

pub const TEXT_FRAGMENT_SHADER: &str = r#"#version 330 core
in vec2 v_texCoord;
in vec2 v_pos;
out vec4 FragColor;

uniform sampler2D u_texture;

// 渐变统一参数
uniform int u_gradient_type; // 0: Solid, 1: Linear, 2: Radial
uniform int u_stop_count;
uniform vec2 u_start;
uniform vec2 u_end;
uniform float u_stops[32];
uniform vec4 u_colors[32];

// 超过 32 个断点的备用方案：数据纹理
uniform bool u_use_texture;
uniform sampler2D u_stop_data;

void main() {
    float alpha = texture(u_texture, v_texCoord).a;
    if (alpha <= 0.0) {
        discard;
    }

    vec4 color;
    if (u_gradient_type == 0 || u_stop_count <= 1) {
        color = u_colors[0];
    } else {
        float t = 0.0;
        if (u_gradient_type == 1) {
            vec2 dir = u_end - u_start;
            float lenSq = dot(dir, dir);
            if (lenSq > 0.00001) {
                t = dot(v_pos - u_start, dir) / lenSq;
            }
        } else if (u_gradient_type == 2) {
            float dist = distance(v_pos, u_start);
            float radius = u_end.x;
            if (radius > 0.00001) {
                t = dist / radius;
            }
        }
        t = clamp(t, 0.0, 1.0);

        if (u_use_texture) {
            color = texture(u_stop_data, vec2(t, 0.5));
        } else {
            color = u_colors[0];
            for (int i = 0; i < u_stop_count - 1; i++) {
                if (t >= u_stops[i] && t <= u_stops[i+1]) {
                    float dt = u_stops[i+1] - u_stops[i];
                    float factor = 0.0;
                    if (dt > 0.00001) {
                        factor = (t - u_stops[i]) / dt;
                    }
                    color = mix(u_colors[i], u_colors[i+1], factor);
                    break;
                } else if (t > u_stops[i+1]) {
                    color = u_colors[i+1];
                }
            }
        }
    }

    FragColor = vec4(color.rgb, color.a * alpha);
}
"#;

pub const BLUR_FRAGMENT_SHADER: &str = r#"#version 330 core
in vec2 v_pos;
out vec4 FragColor;

uniform sampler2D u_texture;
uniform vec2 u_resolution;       // 纹理的分辨率
uniform float u_blurRadius;      // 对应一侧 Sigma 或降采半径
uniform vec2 u_direction;        // 方向 (1.0, 0.0) 或 (0.0, 1.0) 用于 Two-Pass 分离

void main() {
    vec2 texCoord = gl_FragCoord.xy / u_resolution;
    if (u_blurRadius <= 0.0) {
        FragColor = texture(u_texture, texCoord);
        return;
    }

    // Two-Pass 线性高斯近似
    // 根据 CSS 规范，blur_radius 通常对应于 sigma = blur_radius / 2.0
    float sigma = max(u_blurRadius * 0.5, 0.5);
    
    // 为了让高斯模糊平滑衰减，采样窗口至少应覆盖 2.5 ~ 3 个 sigma
    int window = int(ceil(sigma * 2.5));
    // 单轴采样最高 64 个，防止过度影响性能
    if (window > 64) window = 64;

    vec4 color = vec4(0.0);
    float totalWeight = 0.0;

    for (int i = -window; i <= window; i++) {
        float fI = float(i);
        float weight = exp(-(fI * fI) / (2.0 * sigma * sigma));
        
        vec2 offset = fI * u_direction / u_resolution;
        vec2 sampleUV = texCoord + offset;
        
        sampleUV = clamp(sampleUV, 0.0, 1.0);

        vec4 sampled = texture(u_texture, sampleUV);
        color.rgb += sampled.rgb * weight;
        color.a += sampled.a * weight;
        totalWeight += weight;
    }
    
    // 除以高斯分布总权重
    FragColor = vec4(color.rgb / max(totalWeight, 0.0001), color.a / max(totalWeight, 0.0001));
}
"#;
