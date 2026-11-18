from . import twisty_puzzles_py
from .twisty_puzzles_py import Cube3x3 as Cube3x3

__doc__ = twisty_puzzles_py.__doc__
if hasattr(twisty_puzzles_py, "__all__"):
    __all__: list[str] = []
    __all__.extend(twisty_puzzles_py.__all__)
