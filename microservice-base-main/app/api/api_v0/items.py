from fastapi import APIRouter, Depends, HTTPException, status
from fastapi_pagination import PaginationParams, Page
from sqlalchemy.orm import Session

from app import schemas, crud
from app.api import deps
from app.core.security import get_current_user, CurrentUser


items_router = APIRouter()


@items_router.post("/items", response_model=schemas.Item)
def create_item(*,
        db: Session = Depends(deps.get_db),
        item_in: schemas.ItemCreate,
        current_user: CurrentUser=Depends(get_current_user)
        ):
    if not current_user.is_superuser:
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail=f"Only superusers can create items."
        )
    item = crud.get_item_by_name(db, name=item_in.name)
    if item:
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail=f"Item with name: {item_in.name} already exists."
        )
    item = crud.create_item(db, item_in)
    return item


@items_router.get("/items", response_model=Page[schemas.Item])
def get_items(*,
        db: Session = Depends(deps.get_db),
        params: PaginationParams = Depends(),
        ):
    items = crud.get_items(db, params=params)
    return items


@items_router.put("/items/{item_id}", response_model=schemas.Item)
def update_item(*,
        db: Session = Depends(deps.get_db),
        item_id: int,
        item_in: schemas.ItemUpdate,
        current_user: CurrentUser=Depends(get_current_user)
        ):
    if not (current_user.is_superuser or current_user.is_teacher):
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail=f"Only superusers or teachers can edit items."
        )
    item = crud.get_item(db, item_id)
    if not item:
        raise HTTPException(
            status_code=400,
            detail=f"Item with id: {item_id} does not exist."
        )
    item = crud.update_item(db, db_item=item, item_in=item_in)
    return item
