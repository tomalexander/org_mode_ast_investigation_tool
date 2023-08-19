IMAGE_NAME:=org-investigation
# REMOTE_REPO:=harbor.fizz.buzz/private

.PHONY: all
all: build push

.PHONY: build
build:
	docker build -t $(IMAGE_NAME) -f Dockerfile ../

.PHONY: push
push:
ifdef REMOTE_REPO
	docker tag $(IMAGE_NAME) $(REMOTE_REPO)/$(IMAGE_NAME)
	docker push $(REMOTE_REPO)/$(IMAGE_NAME)
else
	@echo "REMOTE_REPO not defined, not pushing to a remote repo."
endif

.PHONY: clean
clean:
	docker rmi $(IMAGE_NAME)
ifdef REMOTE_REPO
	docker rmi $(REMOTE_REPO)/$(IMAGE_NAME)
else
	@echo "REMOTE_REPO not defined, not removing from remote repo."
endif
