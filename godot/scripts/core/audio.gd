## 音频中枢 —— Autoload "Audio"。
## 一个 SFX 播放池 + 一个循环环境音。任意脚本调用 Audio.play_xxx()。
extends Node

const POOL := 6

var _swing: AudioStream = load("res://assets/audio/swing.wav")
var _hit: AudioStream = load("res://assets/audio/hit.wav")
var _pickup: AudioStream = load("res://assets/audio/pickup.wav")
var _step: AudioStream = load("res://assets/audio/step.wav")
var _ambient: AudioStream = load("res://assets/audio/ambient.wav")

var _sfx_players: Array[AudioStreamPlayer] = []
var _next: int = 0
var _ambient_player: AudioStreamPlayer

func _ready() -> void:
	for i in range(POOL):
		var p := AudioStreamPlayer.new()
		add_child(p)
		_sfx_players.append(p)
	_ambient_player = AudioStreamPlayer.new()
	add_child(_ambient_player)
	if _ambient is AudioStreamWAV:
		(_ambient as AudioStreamWAV).loop_mode = AudioStreamWAV.LOOP_FORWARD
	_ambient_player.stream = _ambient
	_ambient_player.volume_db = -8.0

func _play(stream: AudioStream, vol_db: float = 0.0, pitch: float = 1.0) -> void:
	if stream == null:
		return
	var p := _sfx_players[_next]
	_next = (_next + 1) % POOL
	p.stream = stream
	p.volume_db = vol_db
	p.pitch_scale = pitch
	p.play()

func play_swing() -> void:
	_play(_swing, -5.0, randf_range(0.95, 1.08))

func play_hit() -> void:
	_play(_hit, -2.0, randf_range(0.92, 1.06))

func play_pickup() -> void:
	_play(_pickup, -3.0, randf_range(0.98, 1.04))

func play_step() -> void:
	_play(_step, -12.0, randf_range(0.9, 1.12))

func start_ambient() -> void:
	if not _ambient_player.playing:
		_ambient_player.play()

func stop_ambient() -> void:
	_ambient_player.stop()
