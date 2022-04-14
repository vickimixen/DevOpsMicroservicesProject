"""Final router for API v0."""
from fastapi import APIRouter

from .email import email_router

api_v0_router = APIRouter()

api_v0_router.include_router(email_router, tags=["email"])
