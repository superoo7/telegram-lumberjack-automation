import inference
import time
from pynput.mouse import Button, Controller as MouseController
from pynput.keyboard import Controller as KeyboardController, Key
from screenshot import screenshot, region

mouse = MouseController()
keyboard = KeyboardController()
model = inference.get_model("lumberjack/1")

def start_game():
    mouse.position = (210, 640)
    mouse.click(Button.left, 2)
    time.sleep(0.5)
    mouse.click(Button.left, 2)

delay_time = 0.07
def left():
    keyboard.press(Key.left)
    time.sleep(delay_time)
    keyboard.press(Key.left)
    time.sleep(delay_time)

def right():
    keyboard.press(Key.right)
    time.sleep(delay_time)
    keyboard.press(Key.right)
    time.sleep(delay_time)

start_game()

start_time = time.time()
print(f'Elapsed time: {time.time() - start_time} seconds')

image_path = "./tmp/p.png"
while True:
    screenshot(image_path, region=region)
    result = model.infer(image=image_path)
    try: 
        predictions = result[0].predictions[0]
        classname = predictions.class_name
        confidence = predictions.confidence
        print(classname, confidence)
        if (confidence < 0.75):
            continue
        if (classname == "right"):
            right()
            time.sleep(0.05)
        elif (classname == "left"):
            left()
            time.sleep(0.05)
        elif (classname == "restart"):
            start_game()
            right()
            time.sleep(0.1)

    except Exception as e:
        pass

