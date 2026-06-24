## Boss 血条 —— 顶部居中，Boss 激活时显示，名字 + 血条。
extends Control

@onready var name_label: Label = $Panel/VBox/Name
@onready var bar: ProgressBar = $Panel/VBox/Bar

func _ready() -> void:
	visible = false

func bind(boss: Boss, display_name: String) -> void:
	name_label.text = display_name
	bar.max_value = boss.max_hp
	bar.value = boss.hp
	boss.health_changed.connect(_on_health_changed)
	boss.died.connect(_on_died)
	visible = true

func _on_health_changed(current: float, maxhp: float) -> void:
	bar.max_value = maxhp
	bar.value = current

func _on_died() -> void:
	visible = false
