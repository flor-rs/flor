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

void main() {
    vec4 texColor = texture(u_texture, v_texCoord);
    FragColor = texColor * vec4(1.0, 1.0, 1.0, u_opacity);
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
    // 对标准的高分辨率 UI 模糊，sigma 大概是 radius / 3.0，
    // 我们强制限制单 Pass 的采样数量在一个常量内，比如 11 个或更大的定值。
    // 但是要修复边缘发黑现象，我们要处理纹理采样的 clamp 特性，但在 shader 内手动处理较好
    float sigma = max(u_blurRadius * 0.4, 1.0);
    int window = int(ceil(u_blurRadius));
    // 出于性能考虑，单轴的上限也卡住在最高 32 个 sample
    if (window > 32) window = 32;

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

// ==================== FXAA 后处理管线 ====================

/// 全屏三角形顶点着色器 (无需 VBO，用 gl_VertexID 生成)
pub const FULLSCREEN_VERTEX_SHADER: &str = r#"#version 330 core

out vec2 v_texCoord;

void main() {
    // 用 gl_VertexID 生成一个覆盖全屏的大三角形
    // ID=0 -> (-1, -1), ID=1 -> (3, -1), ID=2 -> (-1, 3)
    float x = float((gl_VertexID & 1) << 2) - 1.0;
    float y = float((gl_VertexID & 2) << 1) - 1.0;
    v_texCoord = vec2((x + 1.0) * 0.5, (y + 1.0) * 0.5);
    gl_Position = vec4(x, y, 0.0, 1.0);
}
"#;

/// FXAA 3.11 Quality 简化实现
pub const FXAA_FRAGMENT_SHADER: &str = r#"#version 330 core

in vec2 v_texCoord;
out vec4 FragColor;

uniform sampler2D u_texture;
uniform vec2 u_texelSize; // 1.0 / textureSize

// 将颜色转换为亮度
float luminance(vec3 color) {
    return dot(color, vec3(0.299, 0.587, 0.114));
}

void main() {
    // 当前像素亮度
    vec3 colorM = texture(u_texture, v_texCoord).rgb;
    float lumaM = luminance(colorM);

    // 上下左右四个邻居的亮度
    float lumaU = luminance(texture(u_texture, v_texCoord + vec2(0.0, u_texelSize.y)).rgb);
    float lumaD = luminance(texture(u_texture, v_texCoord - vec2(0.0, u_texelSize.y)).rgb);
    float lumaL = luminance(texture(u_texture, v_texCoord - vec2(u_texelSize.x, 0.0)).rgb);
    float lumaR = luminance(texture(u_texture, v_texCoord + vec2(u_texelSize.x, 0.0)).rgb);

    // 亮度范围
    float lumaMin = min(lumaM, min(min(lumaU, lumaD), min(lumaL, lumaR)));
    float lumaMax = max(lumaM, max(max(lumaU, lumaD), max(lumaL, lumaR)));
    float lumaRange = lumaMax - lumaMin;

    // 如果对比度太低，不需要抗锯齿
    float FXAA_EDGE_THRESHOLD     = 0.125;
    float FXAA_EDGE_THRESHOLD_MIN = 0.0625;
    if (lumaRange < max(FXAA_EDGE_THRESHOLD_MIN, lumaMax * FXAA_EDGE_THRESHOLD)) {
        FragColor = vec4(colorM, 1.0);
        return;
    }

    // 四个对角线邻居的亮度
    float lumaUL = luminance(texture(u_texture, v_texCoord + vec2(-u_texelSize.x, u_texelSize.y)).rgb);
    float lumaUR = luminance(texture(u_texture, v_texCoord + vec2(u_texelSize.x, u_texelSize.y)).rgb);
    float lumaDL = luminance(texture(u_texture, v_texCoord + vec2(-u_texelSize.x, -u_texelSize.y)).rgb);
    float lumaDR = luminance(texture(u_texture, v_texCoord + vec2(u_texelSize.x, -u_texelSize.y)).rgb);

    // 计算子像素混合因子
    float lumaUDLR = lumaU + lumaD + lumaL + lumaR;
    float lumaDiag = lumaUL + lumaUR + lumaDL + lumaDR;
    float subpixFactor = abs((lumaUDLR * 2.0 + lumaDiag) / 12.0 - lumaM);
    subpixFactor = clamp(subpixFactor / lumaRange, 0.0, 1.0);
    subpixFactor = smoothstep(0.0, 1.0, subpixFactor);
    float subpixBlend = subpixFactor * subpixFactor * 0.15;

    // 确定边缘方向
    float edgeH = abs(-2.0 * lumaL + lumaUL + lumaDL) +
                  abs(-2.0 * lumaM + lumaU  + lumaD) * 2.0 +
                  abs(-2.0 * lumaR + lumaUR + lumaDR);
    float edgeV = abs(-2.0 * lumaU + lumaUL + lumaUR) +
                  abs(-2.0 * lumaM + lumaL  + lumaR) * 2.0 +
                  abs(-2.0 * lumaD + lumaDL + lumaDR);
    bool isHorizontal = (edgeH >= edgeV);

    // 沿法线方向的步长
    float stepLen = isHorizontal ? u_texelSize.y : u_texelSize.x;
    float gradientPos, gradientNeg;
    if (isHorizontal) {
        gradientPos = lumaU - lumaM;
        gradientNeg = lumaD - lumaM;
    } else {
        gradientPos = lumaR - lumaM;
        gradientNeg = lumaL - lumaM;
    }
    bool pIsLarger = abs(gradientPos) >= abs(gradientNeg);
    float gradientScaled = 0.25 * max(abs(gradientPos), abs(gradientNeg));

    // 沿着边缘方向搜索端点
    vec2 edgeStep = isHorizontal ? vec2(u_texelSize.x, 0.0) : vec2(0.0, u_texelSize.y);
    vec2 edgeUV = v_texCoord;
    if (isHorizontal) {
        edgeUV.y += pIsLarger ? (stepLen * 0.5) : (-stepLen * 0.5);
    } else {
        edgeUV.x += pIsLarger ? (stepLen * 0.5) : (-stepLen * 0.5);
    }

    float edgeLuma = (pIsLarger ? (lumaM + lumaU) : (lumaM + lumaD)) * 0.5;
    if (!isHorizontal) {
        edgeLuma = (pIsLarger ? (lumaM + lumaR) : (lumaM + lumaL)) * 0.5;
    }

    // 向两个方向搜索，找到边缘结束的位置
    vec2 uvPos = edgeUV + edgeStep;
    vec2 uvNeg = edgeUV - edgeStep;
    float lumaPosEnd = luminance(texture(u_texture, uvPos).rgb) - edgeLuma;
    float lumaNegEnd = luminance(texture(u_texture, uvNeg).rgb) - edgeLuma;
    bool reachedPos = abs(lumaPosEnd) >= gradientScaled;
    bool reachedNeg = abs(lumaNegEnd) >= gradientScaled;

    const int SEARCH_STEPS = 10;
    for (int i = 2; i < SEARCH_STEPS; i++) {
        if (!reachedPos) {
            uvPos += edgeStep;
            lumaPosEnd = luminance(texture(u_texture, uvPos).rgb) - edgeLuma;
            reachedPos = abs(lumaPosEnd) >= gradientScaled;
        }
        if (!reachedNeg) {
            uvNeg -= edgeStep;
            lumaNegEnd = luminance(texture(u_texture, uvNeg).rgb) - edgeLuma;
            reachedNeg = abs(lumaNegEnd) >= gradientScaled;
        }
        if (reachedPos && reachedNeg) break;
    }

    // 计算沿边缘的混合比例
    float distPos, distNeg;
    if (isHorizontal) {
        distPos = uvPos.x - v_texCoord.x;
        distNeg = v_texCoord.x - uvNeg.x;
    } else {
        distPos = uvPos.y - v_texCoord.y;
        distNeg = v_texCoord.y - uvNeg.y;
    }
    float distMin = min(distPos, distNeg);
    float edgeLen = distPos + distNeg;
    float edgeBlend = 0.5 - distMin / edgeLen;

    // 根据亮度差的符号决定是否反方向偏移
    bool correctSign = ((distPos < distNeg ? lumaPosEnd : lumaNegEnd) < 0.0) != (lumaM - edgeLuma < 0.0);
    float finalEdgeBlend = correctSign ? edgeBlend : 0.0;

    float finalBlend = max(subpixBlend, finalEdgeBlend);

    // 沿法线方向偏移采样
    vec2 finalUV = v_texCoord;
    if (isHorizontal) {
        finalUV.y += (pIsLarger ? 1.0 : -1.0) * finalBlend * stepLen;
    } else {
        finalUV.x += (pIsLarger ? 1.0 : -1.0) * finalBlend * stepLen;
    }

    FragColor = vec4(texture(u_texture, finalUV).rgb, 1.0);
}
"#;
