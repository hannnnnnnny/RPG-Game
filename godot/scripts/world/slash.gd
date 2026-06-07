## 剑光月牙效果 —— 短时间动画后销毁
extends Node2D

const DURATION := 0.18
var elapsed := 0.0

func _process(delta: float) -> void:
	elapsed += delta
	if elapsed >= DURATION:
		queue_free()
		return
	queue_redraw()

func _draw() -> void:
	var t: float = clamp(elapsed / DURATION, 0.0, 1.0)
	# Crescent arc — sweep angle range expands then fades
	var center := Vector2.ZERO
	var r_outer := 64.0
	var r_inner := 48.0
	var half_arc: float = PI / 2.4

	# Determine visible arc range based on t (Stardew-style sweep)
	var seg_start: float
	var seg_end: float
	if t < 0.25:
		seg_start = -half_arc
		seg_end = -half_arc + (t / 0.25) * (half_arc * 0.5)
	elif t < 0.7:
		var p := (t - 0.25) / 0.45
		seg_start = -half_arc
		seg_end = -half_arc * 0.5 + p * (half_arc * 1.5)
	else:
		var p := (t - 0.7) / 0.3
		seg_start = -half_arc + p * (half_arc * 1.8)
		seg_end = half_arc

	# Alpha fade
	var alpha := 1.0
	if t > 0.6:
		alpha = 1.0 - (t - 0.6) / 0.4

	var step := 0.04
	var a := seg_start
	while a <= seg_end:
		var edge_t: float = min((a - seg_start) / 0.35, (seg_end - a) / 0.35, 1.0)
		var local_alpha: float = alpha * max(0.2, edge_t)
		for r in range(int(r_inner), int(r_outer), 2):
			var x: float = center.x + cos(a) * r
			var y: float = center.y + sin(a) * r
			var t_core: float = (r - r_inner) / (r_outer - r_inner)
			var color: Color
			if t_core > 0.78:
				color = Color(1, 1, 1, local_alpha)
			elif t_core > 0.45:
				color = Color(0.86, 0.91, 1, local_alpha * 0.75)
			else:
				color = Color(0.59, 0.71, 0.9, local_alpha * 0.32)
			draw_rect(Rect2(x, y, 2, 2), color, true)
		a += step
