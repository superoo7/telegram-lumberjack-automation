import Quartz
import LaunchServices
from Cocoa import NSURL
import Quartz.CoreGraphics as CG


def screenshot(path, region = None):
    """region should be a CGRect, something like:

    >>> import Quartz.CoreGraphics as CG
    >>> region = CG.CGRectMake(0, 0, 100, 100)
    >>> sp = ScreenPixel()
    >>> sp.capture(region=region)

    The default region is CG.CGRectInfinite (captures the full screen)
    """

    if region is None:
        region = CG.CGRectInfinite

    # Create screenshot as CGImage
    image = CG.CGWindowListCreateImage(
        region,
        CG.kCGWindowListOptionOnScreenOnly,
        CG.kCGNullWindowID,
        CG.kCGWindowImageDefault)

    dpi = 72 # FIXME: Should query this from somewhere, e.g for retina displays

    url = NSURL.fileURLWithPath_(path)

    dest = Quartz.CGImageDestinationCreateWithURL(
        url,
        LaunchServices.kUTTypePNG, # file type
        1, # 1 image in file
        None
        )

    properties = {
        Quartz.kCGImagePropertyDPIWidth: dpi,
        Quartz.kCGImagePropertyDPIHeight: dpi,
        }

    # Add the image to the destination, characterizing the image with
    # the properties dictionary.
    Quartz.CGImageDestinationAddImage(dest, image, properties)

    # When all the images (only 1 in this example) are added to the destination, 
    # finalize the CGImageDestination object. 
    Quartz.CGImageDestinationFinalize(dest)


region = CG.CGRectMake(0, 25, 420, 725)

if __name__ == '__main__':
    import time
    start_time = time.time()
    screenshot("./tmp/testscreenshot_partial.png", region=region)
    print(f'Elapsed time: {time.time() - start_time} seconds')