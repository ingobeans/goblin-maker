from flask import Flask
from flask import request
import os,time

app = Flask(__name__)

file_path = os.path.realpath(__file__)
levels_path = os.path.join(os.path.abspath(os.path.join(file_path, os.pardir)),"levels")

if not os.path.isdir(levels_path):
    os.mkdir(levels_path)

@app.route("/")
def home():
    return "<p>hello! this is the backend server for my videogame called 'Goblin Maker'</p>\n<p>you can try it <a href='https://github.com/ingobeans/goblin-maker'>here</a></p>"
@app.route("/list")
def list():
    return ",".join(os.listdir(levels_path))
@app.route("/get/<id>")
def get(id: str):
    if not all([c.isalnum() or c == ' ' for c in id]):
        return "invalid :("
    path = os.path.join(levels_path,id)
    if not os.path.isfile(path):
        return "level doesn't exist :("
    with open(path,"r") as f:
        return f.read()
@app.route("/upload/<id>")
def upload(id: str):
    if not all([c.isalnum() or c == ' ' for c in id]):
        return "invalid :("
    data = request.headers.get("data")
    if data is None:
        return "missing data! :("

    path = os.path.join(levels_path,id)
    if os.path.isfile(path):
        return "level name taken! :("
    with open(path,"w") as f:
        f.write(data)
    return "ok"
app.run(port=5462)