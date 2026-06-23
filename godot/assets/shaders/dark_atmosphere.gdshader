// 暗黑氛围后处理 —— 全屏 canvas_item shader。
// 挂在一个铺满屏幕的 ColorRect 上（在世界之上、UI 之下的 CanvasLayer），
// 读取 screen_texture 后做四件事：
//   1. 降饱和（让画面褪色，像被黑潮侵蚀）
//   2. 暗部染紫（阴影偏冷紫，高光保留暖色 —— "暗黑星露谷"的核心调子）
//   3. 暗角 vignette（聚焦视线，营造矿洞封闭感）
//   4. 污染渗透（corruption 越高，边缘紫色越浓，由脚本动态传入）
shader_type canvas_item;

uniform sampler2D screen_tex : hint_screen_texture, filter_linear_mipmap;
uniform float corruption : hint_range(0.0, 1.0) = 0.05;
uniform float desaturation : hint_range(0.0, 1.0) = 0.22;
uniform float vignette_strength : hint_range(0.0, 1.0) = 0.5;

void fragment() {
	vec3 col = texture(screen_tex, SCREEN_UV).rgb;

	// 1. 降饱和
	float lum = dot(col, vec3(0.299, 0.587, 0.114));
	col = mix(col, vec3(lum), desaturation);

	// 2. 暗部染紫：亮度低的像素往冷紫推，亮的保留
	vec3 shadow_grade = col * vec3(0.72, 0.64, 0.86);
	col = mix(shadow_grade, col, smoothstep(0.0, 0.45, lum));

	// 3. 暗角
	vec2 d = SCREEN_UV - vec2(0.5);
	float dist = length(d);
	float vig = smoothstep(0.85, 0.25, dist);
	col *= mix(1.0 - vignette_strength, 1.0, vig);

	// 4. 污染渗透：边缘紫色 bleed，随 corruption 增强
	float edge = 1.0 - vig;
	vec3 corrupt_tint = vec3(0.42, 0.06, 0.58);
	col = mix(col, col + corrupt_tint, edge * corruption * 0.9);

	COLOR = vec4(col, 1.0);
}
