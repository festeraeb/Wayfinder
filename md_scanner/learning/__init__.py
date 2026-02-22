# Adaptive Learning System
# Tracks user behavior, detects difficulties, suggests improvements, fades as user masters skills

from .behavior_tracker import BehaviorTracker
from .difficulty_detector import DifficultyDetector
from .suggestion_engine import SuggestionEngine
from .adaptive_coach import AdaptiveCoach

__all__ = [
    "BehaviorTracker",
    "DifficultyDetector",
    "SuggestionEngine",
    "AdaptiveCoach",
]
