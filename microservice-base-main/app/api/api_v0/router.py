"""Final router for API v0."""
from fastapi import APIRouter

from .items import items_router

api_v0_router = APIRouter()

api_v0_router.include_router(items_router, tags=["Items"])
