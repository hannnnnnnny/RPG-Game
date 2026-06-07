## 永久选择面板 —— 居中模态对话框
extends Control

@onready var title_label: Label = $Panel/VBox/Title
@onready var body_label: Label = $Panel/VBox/Body
@onready var options_container: VBoxContainer = $Panel/VBox/Options

func _ready() -> void:
	visible = false
	GameState.choice_opened.connect(_on_choice_opened)
	GameState.choice_closed.connect(_on_choice_closed)

func _on_choice_opened(choice: Dictionary) -> void:
	title_label.text = choice.title
	body_label.text = choice.body
	# Clear old options
	for c in options_container.get_children():
		c.queue_free()
	for opt in choice.options:
		var btn := Button.new()
		btn.custom_minimum_size = Vector2(0, 60)
		btn.text = "%s\n%s" % [opt.label, opt.description]
		btn.alignment = HORIZONTAL_ALIGNMENT_LEFT
		btn.pressed.connect(_on_option_picked.bind(opt.id))
		options_container.add_child(btn)
	visible = true

func _on_choice_closed() -> void:
	visible = false

func _on_option_picked(option_id: String) -> void:
	var corruption_delta: int
	var sanity_value: int
	match option_id:
		"save":
			sanity_value = 82
			corruption_delta = 5
		"abandon":
			sanity_value = 70
			corruption_delta = 8
		"kill":
			sanity_value = 64
			corruption_delta = 13
		_:
			sanity_value = 70
			corruption_delta = 5

	var approved: bool = GameState.request_state_change({
		"type": "record_first_choice",
		"requested_by": "受伤矮人",
		"target_id": "first_dwarf_choice",
		"reason": "玩家选择：%s" % option_id,
		"effects": [
			{"path": "flags.first_dwarf_choice", "value": option_id},
			{"path": "sanity", "value": sanity_value},
			{"path": "corruption", "value": corruption_delta}
		]
	})

	if approved:
		GameState.add_gold(8 if option_id == "save" else 16)
		var line := ""
		match option_id:
			"save": line = "他还活着。也许这会让灰灯镇多一个问题，也许多一个证人。"
			"abandon": line = "很好。怜悯会让矿道坍得更慢，但不会让你活得更久。"
			"kill": line = "血没有溅到你身上，它像认识你一样避开了。"
		GameState.set_dialogue({
			"speaker": "受伤矮人" if option_id == "save" else "克哈低语",
			"text": line,
			"tone": Types.TONE_MEMORY if option_id == "save" else Types.TONE_WHISPER
		})
	GameState.close_choice()
