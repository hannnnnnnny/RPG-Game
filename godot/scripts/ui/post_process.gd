## 后处理控制 —— 把 GameState 的污染值喂给暗黑氛围 shader。
## 污染越高，屏幕边缘的紫色侵蚀越浓。
extends ColorRect

func _ready() -> void:
	GameState.world_state_changed.connect(_on_world_changed)
	_apply()

func _on_world_changed(_path: String, _value: Variant) -> void:
	_apply()

func _apply() -> void:
	var mat := material as ShaderMaterial
	if mat == null:
		return
	var c: float = float(GameState.world_state.get("corruption", 0)) / 100.0
	mat.set_shader_parameter("corruption", c)
