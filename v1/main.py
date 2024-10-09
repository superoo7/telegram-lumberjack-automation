from time import sleep
import time
import pyautogui

im1 = pyautogui.screenshot()
im1.save("images/screenshot.png")

def handle_click(button_location):
    button = pyautogui.locateOnScreen(button_location, grayscale=True, confidence=0.6)
    buttonx, buttony = pyautogui.center(button)
    pyautogui.moveTo(buttonx, buttony)
    pyautogui.click(buttonx, buttony)
    pyautogui.click(buttonx, buttony)
    return buttonx, buttony

def keyboard_btn_press(key):
    print(f"[{key}]")
    pyautogui.press(key)
    pyautogui.press(key)

start_btn_x, start_btn_y = None, None

print("---Starting Lumberjack---")
try:
    (start_btn_x, start_btn_y) = handle_click("lumber/restart_button.png")
    print("Restart button clicked")
except Exception as e:
    # Play Lumberjack
    handle_click("lumber/play_button.png")
    print("Clicked play button")

    sleep(2)

    # Start Lumberjack
    (start_btn_x, start_btn_y) = handle_click("lumber/start_button.png")
    print("Clicked start button")

print("BTN", start_btn_x, start_btn_y)

# press right twice
keyboard_btn_press("right")
CONFIDENCE = 0.7

def find_branch(image_path):
    now = time.time()
    branch = pyautogui.locateOnScreen(image_path, confidence=CONFIDENCE, grayscale=True, region=[
        start_btn_x - 800, start_btn_y - 800, start_btn_x + 800, start_btn_y
    ])

    print(f"Time taken: {time.time() - now} seconds")
    return branch

while True:
    # View on Screen
    left_branch, right_branch = None, None
    try:
        left_branch = find_branch("lumber/left_branch.png")
        right_branch = find_branch("lumber/right_branch.png")
    except Exception as e:
        try: 
            if not right_branch:  
                right_branch = find_branch("lumber/right_branch.png")
        except Exception as e:
            pass
    
    if left_branch and right_branch:
        print("[BOTH]")
        left_branch_y = left_branch.top + left_branch.height
        right_branch_y = right_branch.top + right_branch.height
        if left_branch_y > right_branch_y:
            keyboard_btn_press("right")
        else:
            keyboard_btn_press("left")
    elif left_branch:
        keyboard_btn_press("right")
    elif right_branch:
        keyboard_btn_press("left")