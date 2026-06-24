## 砸地冲击波 —— 一圈快速扩散后消失的环。
extends Node2D

var max_radius: float = 130.0
const DURATION := 0.35
var elapsed: float = 0.0

func _process(delta: float) -> void:
	elapsed += delta
	if elapsed >= DURATION:
		queue_free()
		return
	queue_redraw()

func _draw() -> void:
	var t: float = clampf(elapsed / DURATION, 0.0, 1.0)
	var r: float = max_radius * t
	var alpha: float = (1.0 - t) * 0.8
	# Pixel ring drawn as short radial dashes.
	var col := Color(0.85, 0.45, 1.0, alpha)
	var step := 0.18
	var a := 0.0
	while a < TAU:
		var x := cos(a) * r
		var y := sin(a) * r
		draw_rect(Rect2(x - 2, y - 2, 4, 4), col, true)
		a += step
