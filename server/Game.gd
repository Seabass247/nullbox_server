extends Spatial

# Declare member variables here. Examples:
# var a = 2
# var b = "text"
var laminar
var players: Dictionary
var time: float

# Called when the node enters the scene tree for the first time.
func _ready():
	laminar = get_node("/root/Laminar")
	laminar.init_server(self as Node, "12345" as String)

func _process(delta):
	time += delta
	#print("Tick: ", time)

func _on_net_player_connected(id: int, data):
	print("Configure new player id= ", id, ", name=", data[1])
	players[id] = data[1]
	spawn_player(id, data[1])
	laminar.send_to(id as int, "/root/MainMenu:server_response" as String, ["success", id])

func _on_net_new_connection(id: int):
	print("New player connected")
	
func _on_net_timed_out(id: int):
	players.erase(id)
	var player_path = "Players/player_" + String(id)
	if has_node(player_path):
		get_node(player_path).free()
		print("Removed timed out player")
	

func _on_net_player_pos(id: int, data):
	var player_path = "Players/player_" + String(id)
	var player: KinematicBody = get_node(player_path)
	#print("player_", id, ": pos=", data[0])
	player.global_transform.origin = data[0]

func spawn_player(id, username):
	var player = load("res://player.tscn")
	var player_instance = player.instance()
	var name = "player_" + String(id)
	player_instance.set_name(name)
	get_node("Players").add_child(player_instance)
	print("Spawn player: ", name)