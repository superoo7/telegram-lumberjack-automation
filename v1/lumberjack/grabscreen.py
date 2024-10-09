import numpy as np
import cv2
import mss
import mss.tools

def grab_screen(region=None):
    with mss.mss() as sct:
        # If a region is specified, capture that portion of the screen
        if region:
            left, top, right, bottom = region
            monitor = {"top": top, "left": left, "width": right - left, "height": bottom - top}
        else:
            # Otherwise, capture the entire screen
            monitor = sct.monitors[1]  # Primary monitor

        # Capture the screen
        screenshot = sct.grab(monitor)
        
        # Convert the captured image to a numpy array (BGR format for OpenCV)
        img = np.array(screenshot)

        # mss returns BGRA by default, we convert it to BGR for OpenCV
        img = cv2.cvtColor(img, cv2.COLOR_BGRA2BGR)
    
    return img

# Example usage:
# img = grab_screen()
# cv2.imshow('Captured Screen', img)
# cv2.waitKey(0)
# cv2.destroyAllWindows()

if __name__ == "__main__":
    try:
        import time
        now = time.time()
        # Capture the entire screen
        full_screen = grab_screen()
        print("Fullscreen Time taken: {:.2f} seconds".format(time.time() - now))
        

        now = time.time()
        # Capture a specific region (example: top-left 500x500 pixels)
        region = (0, 50, 410, 770)
        region_screen = grab_screen(region)
        print("Region Time taken: {:.2f} seconds".format(time.time() - now))


        cv2.imshow('Full Screen Capture', full_screen)
        cv2.imshow('Region Capture', region_screen)
        
        print("Press Ctrl+C to exit")
        while True:
            if cv2.waitKey(1) & 0xFF == ord('q'):
                break
    except KeyboardInterrupt:
        print("\nExiting...")
    finally:
        # Close all windows
        cv2.destroyAllWindows()

