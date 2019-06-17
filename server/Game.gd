extends Spatial

# Declare member variables here. Examples:
# var a = 2
# var b = "text"
var laminar
var players: Dictionary

# Called when the node enters the scene tree for the first time.
func _ready():
	laminar = get_node("/root/Laminar")
	laminar.init_server("12345" as String, self as Node)

func _on_net_player_connected(id: int, data):
	print("New player connected! id= ", id, ", name=", data[1])
	players[id] = data[1]
	spawn_player(id, data[1])
	laminar.send_to(id as int, "/root/MainMenu:server_response" as String, ["success", id])
	
func _on_net_player_pos(id: int, data):
	var player_path = "Players/player_" + String(id)
	var player: KinematicBody = get_node(player_path)
	print("player_", id, ": pos=", data[0])
	player.global_transform.origin = data[0]

func spawn_player(id, username):
	var player = load("res://player.tscn")
	var player_instance = player.instance()
	var name = "player_" + String(id)
	player_instance.set_name(name)
	get_node("Players").add_child(player_instance)
	print("Spawn player: ", name)