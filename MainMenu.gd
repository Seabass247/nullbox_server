extends CanvasLayer

var address
var username
var laminar 
# Called when the node enters the scene tree for the first time.
func _ready():
	get_node("Control/Button").connect("pressed", self, "on_connect")
	var Laminar = load("res://laminar_client.gdns")
	laminar = Laminar.new()
	var path : String = self.get_path()
	print("NODEPATH", path)
	laminar.set_recv_callback(self as CanvasLayer, "laminar_recv")
	laminar.test(".", "goodbyee")
	#laminar.set_recv_callback(self.get_node("."), "laminar_recv")
	
func on_connect():
	address = get_node("Control/ServerAddrBox").text.strip_edges()
	username = get_node("Control/UsernameBox").text
	var global = get_node("/root/Global")
	global.username = username
	global.address = address
	laminar.new(address)
	laminar.send("hah")
	var got_packet: PoolByteArray = laminar.get_packet()
	print("Got packet: [", got_packet.get_string_from_utf8(),"]")
	#get_tree().change_scene("res://Game.tscn")

func laminar_recv(data):
	print("Client got data: ", data)
