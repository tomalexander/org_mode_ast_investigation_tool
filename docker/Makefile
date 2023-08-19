IMAGE_NAME:=org-investigation

.PHONY: build
build:
	docker build -t $(IMAGE_NAME) -f Dockerfile ../

.PHONY: clean
clean:
	docker rmi $(IMAGE_NAME)
