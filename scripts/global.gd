extends Node

var laminar

var username
var network_id

func _ready():
    var Laminar = load("res://laminar_client.gdns")
    laminar = Laminar.new()
    

func init_client(username, address, network_id):
    laminar.new_connection(address)
    laminar.start_receiving(self as Node)
