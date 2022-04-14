"""Final router for API v0."""
from fastapi import APIRouter

from .assignments import assignments_router
from .attached_students import attached_students_router

api_v0_router = APIRouter()

api_v0_router.include_router(assignments_router, tags=["Assignments"])
api_v0_router.include_router(attached_students_router, tags=["Attachments"])
